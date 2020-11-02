#include "structures/framebuffer.h"
#include "structures/list.h"

#ifdef __cplusplus
extern "C" {
#endif

static sFrameBufferID last_framebuffer = 0;
static uint16_t last_framebuffer_textures = 0;

static const GLuint sFrameBufferAttachments[17] = {
	GL_COLOR_ATTACHMENT0,
	GL_COLOR_ATTACHMENT1,
	GL_COLOR_ATTACHMENT2,
	GL_COLOR_ATTACHMENT3,
	GL_COLOR_ATTACHMENT4,
	GL_COLOR_ATTACHMENT5,
	GL_COLOR_ATTACHMENT6,
	GL_COLOR_ATTACHMENT7,
	GL_COLOR_ATTACHMENT8,
	GL_COLOR_ATTACHMENT9,
	GL_COLOR_ATTACHMENT10,
	GL_COLOR_ATTACHMENT11,
	GL_COLOR_ATTACHMENT12,
	GL_COLOR_ATTACHMENT13,
	GL_COLOR_ATTACHMENT14,
	GL_COLOR_ATTACHMENT15,
	GL_DEPTH_ATTACHMENT
};

sFrameBuffer sFrameBufferCreate(uint16_t width, uint16_t height, bool depthbuffer)
{
    sFrameBuffer fb = {
        width, height, 
        0,0,
        {0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0},
        0
    };

    if (depthbuffer) {
        glc(glGenRenderbuffers(1, &fb.renderbuffer_id));
        glc(glBindRenderbuffer(GL_RENDERBUFFER, fb.renderbuffer_id));
        glc(glRenderbufferStorage(GL_RENDERBUFFER, GL_DEPTH_COMPONENT24, width, height));
        glc(glBindRenderbuffer(GL_RENDERBUFFER, 0));
    } else {
        fb.renderbuffer_id = 0;
    }

    glc(glGenFramebuffers(1,&fb.framebuffer_id));
    glc(glBindFramebuffer(GL_FRAMEBUFFER, fb.framebuffer_id));
    glc(glBindFramebuffer(GL_FRAMEBUFFER, 0));

    return fb;
}

void sFrameBufferAddDepth(sFrameBufferID fb)
{
    if (glIsRenderbuffer(fb->renderbuffer_id)) return;
    glc(glGenRenderbuffers(1, &fb->renderbuffer_id));
    glc(glBindRenderbuffer(GL_RENDERBUFFER, fb->renderbuffer_id));
    glc(glRenderbufferStorage(GL_RENDERBUFFER, GL_DEPTH_COMPONENT24, fb->width, fb->height));
    glc(glBindRenderbuffer(GL_RENDERBUFFER, 0));
}

void sFrameBufferRemoveDepth(sFrameBufferID fb)
{
    if (!glIsRenderbuffer(fb->renderbuffer_id)) return;
    glc(glDeleteRenderbuffers(1, &fb->renderbuffer_id));
}

void sFrameBufferAddRenderTarget(sFrameBufferID fb, sTextureID texture)
{
    if (!texture || texture==fb->depth_render_target) return;
    if (sListIndexOf(fb->color_render_targets, texture)==MAX_INDEX)
    {
        sTextureAddFramebufferUser(texture, fb);
        for (int i=0; i<16; i++) {
            if (!fb->color_render_targets[i]) {
                fb->color_render_targets[i] = texture;
                break;
            }
        }
    }
}

void sFrameBufferRemoveRenderTarget(sFrameBufferID fb, sTextureID texture)
{
    for (size_t i=0; i<16; i++)
    {
        if (fb->color_render_targets[i] && fb->color_render_targets[i]==texture)
        {
            sListPopItem(texture->framebuffer_users, fb);
            fb->color_render_targets[i] = 0;
            break;
        }
    }
}

void sFrameBufferRemoveRenderTargetIndex(sFrameBufferID fb, int texture)
{
    if (texture<16 && fb->color_render_targets[texture]) {
        sListPopItem(fb->color_render_targets[texture]->framebuffer_users, fb);
        fb->color_render_targets[texture] = 0;
    }
}

void sFrameBufferSetDepthTarget(sFrameBufferID fb, sTextureID texture)
{
    if (sListIndexOf(fb->color_render_targets, texture) != MAX_INDEX) {
        return;
    }
    if (texture) {
        if (texture->type != GL_TEXTURE_2D &&
            texture->type != GL_TEXTURE_CUBE_MAP_POSITIVE_X &&
            texture->type != GL_TEXTURE_CUBE_MAP_NEGATIVE_X &&
            texture->type != GL_TEXTURE_CUBE_MAP_POSITIVE_Y &&
            texture->type != GL_TEXTURE_CUBE_MAP_NEGATIVE_Y &&
            texture->type != GL_TEXTURE_CUBE_MAP_POSITIVE_Z &&
            texture->type != GL_TEXTURE_CUBE_MAP_NEGATIVE_Z) {
                return;
        }
        sListPopItem(fb->depth_render_target->framebuffer_users, fb);
    }
    fb->depth_render_target = texture;
    if (sListIndexOf(texture->framebuffer_users, fb) == MAX_INDEX) {
        sListPushBack(texture->framebuffer_users, fb);
    }
}

void sFrameBufferBind(sFrameBufferID fb, uint16_t textures)
{
    if (fb == last_framebuffer && textures==last_framebuffer_textures) return;
    if (!fb) {
        last_framebuffer = 0;
        glc(glBindFramebuffer(GL_FRAMEBUFFER, 0));
        return;
    }
    glc(glViewport(0,0,fb->width, fb->height));
    if (!glIsFramebuffer(fb->framebuffer_id)) {
        if (fb->framebuffer_id) {
            fprintf(stderr, "%ud is not a framebuffer (%hux%hu)\n", fb->framebuffer_id, fb->width, fb->height);
            puts((const char*)0);
        } else {
            glc(glBindFramebuffer(GL_FRAMEBUFFER, 0));
            last_framebuffer = 0;
        }
        return;
    }
    //printf("Framebuffer (%hux%hu)\n", fb->width, fb->height);
    glc(glBindFramebuffer(GL_FRAMEBUFFER, fb->framebuffer_id));
    
    if (fb->renderbuffer_id) {
        glc(glEnable(GL_DEPTH_TEST));
        glc(glDepthFunc(GL_LEQUAL));
        glc(glFramebufferRenderbuffer(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_RENDERBUFFER, fb->renderbuffer_id));
    } else {
        glc(glDisable(GL_DEPTH_TEST));
    }
    size_t i, att=0;
    //printf("Binding ");
    for (i=0; i<16; i++) {
        if ((textures&(1<<i)) && fb->color_render_targets[i]) {
            //printf("Binding %s\n", fb->color_render_targets[i]->name);
            glc(glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0 + att, fb->color_render_targets[i]->type, fb->color_render_targets[i]->ID, 0));
            att++;
        }
    }
    
    //printf("\nglDrawBuffers(%lu, sFrameBufferAttachments)\n", att);
    glc(glDrawBuffers(att, sFrameBufferAttachments));
    last_framebuffer = fb;
    last_framebuffer_textures = textures;
}

void sFrameBufferFillColor(sColor rgba)
{
    glc(glClearColor(rgba.r, rgba.b, rgba.b, rgba.a));
    if (last_framebuffer && last_framebuffer->framebuffer_id)
    {
    	glc(glBindFramebuffer(GL_FRAMEBUFFER, last_framebuffer->framebuffer_id));
    	bool sd = last_framebuffer->renderbuffer_id!=0;
        glc(glClear(GL_COLOR_BUFFER_BIT | (GL_DEPTH_BUFFER_BIT * sd)));
    } else {
    	//puts("Clearing default frame buffer");
        glc(glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT));
    }
}

void sFrameBufferDelete(sFrameBufferID fb)
{
    for (size_t i=0; i<16; i++) {
        if (fb->color_render_targets[i]) {
            sListPopItem(fb->color_render_targets[i]->framebuffer_users, fb);
        }
    }
    
    if (glIsFramebuffer(fb->framebuffer_id)) {
        glc(glDeleteFramebuffers(1, &fb->framebuffer_id));
    }
    if (glIsRenderbuffer(fb->renderbuffer_id)) {
        glc(glDeleteRenderbuffers(1, &fb->renderbuffer_id));
    }
    fb->width = 0;
    fb->height = 0;
    if (fb==last_framebuffer) {
        sFrameBufferBind(0, 0);
    }
}

#ifdef __cplusplus
}
#endif
