/*
 * window.c
 *
 *  Created on: 22 дек. 2017 г.
 *      Author: ivan
 */

#include "engine.h"
#include "locales.h"
#include "2D_renderer/2D_renderer.h"

//#define THREADS

#ifdef THREADS
#include "pthread.h"
#endif

static _Bool glstarted = 0;
static GLFWwindow* s_pWindow;

sScene* active_scene;

// render.c externs //
extern sTexture noise_texture;
extern _Bool _renderApplyChanges;
//////////////////////

//static sShader *shaderList[4] = {&shader,&skin_shader,&shader_depth,&skin_shader_depth};
static double sFrameTime = 0.0166666666;

/*static double sLogicTime = 0.0166666666;
static double sPhysicsTime = 0.0166666666;
static double sRenderTime = 0.0166666666;*/

static double sFrameAverangeTime = 0.0166666666;
static double sLogicAverangeTime = 0.0166666666;
static double sPhysicsAverangeTime = 0.0166666666;
static double sRenderAverangeTime = 0.0166666666;
static double sTime = 0.0;

struct
{
	double animations;
	double scripts;
	double physics;
	double placing;
	double shadows;
	double rasterizing;
	double shading;
	double interface;
	double buffers;
} sProfilingTimers;

//static double sFrameAverangeTimeAccum = 0.0;
/*static double sLogicAverangeTimeAccum = 0.0;
static double sPhysicsAverangeTimeAccum = 0.0;
static double sRenderAverangeTimeAccum = 0.0;*/

static double frame_input_time = 0.0166666666;
static char sDebugString[1024];
size_s bytes_allocated = 0;

double sEngineGetTime(void)
{
	return sTime;
}

void sEngineSetActiveScene(sScene* scene)
{
	active_scene = scene;
}

void sRenderDeferred(int val)
{
	_renderDeferred = val;
}

void sRenderBloomOn(void)
{
	_renderBloom = 1;
}

void sRenderBloomOff(void)
{
	_renderBloom = 0;
}

void sRenderVectorsOn(void)
{
	_renderVectors = 1;
}

void sRenderVectorsOff(void)
{
	_renderVectors = 0;
}

void sRenderRayTracingOn(void)
{
	_renderRayTracing = 1;
}

void sRenderRayTracingOff(void)
{
	_renderRayTracing = 0;
}

_Bool useProgram(GLuint id)
{
    _Bool res = id!=activeShader;
    if (id!=activeShader)
    {
        activeShader=id;
        glUseProgram(id);
        glGetError();
    }
    return res;
}

/*
void S_EventFunc(GLFWwindow* pWindow, int key, int scancode, int action, int mods)
{
    sStackPol(&keyStack,scancode | (action<<16));
    return;
    printf("keyboard scan code 0x%08X %s%s%s%s",scancode,
           ((mods&1) ? " + shift":""),
           ((mods&2) ? " + ctrl":""),
           ((mods&3) ? " + alt":""),
           ((mods&4) ? " + super":""));
    switch (action)
    {
        case GLFW_RELEASE:	printf(" (rel.)\n");break;
        case GLFW_PRESS:	printf(" (press.)\n");break;
        case GLFW_REPEAT:	printf(" (hold.)\n");break;
    };
}*/

//Input operations
static double cx,cy;
static float pcx,pcy;
static float dcx,dcy;
void sMouseSetPosition(float x,float y)
{
    glfwSetCursorPos(s_pWindow,x,y);
    cx = x;
    cy = y;
    dcx = 0.0;
    dcy = 0.0;
}
void sMouseGetPosition(float* x,float* y)
{
    glfwGetCursorPos(s_pWindow,&cx,&cy);
    *x = cx;
    *y = cy;
}
void sMouseGetDelta(float* x,float* y)
{
    *x = dcx;
    *y = dcy;
}

void sGetMousePosition(float* pos)
{
    glfwGetCursorPos(s_pWindow,&cx,&cy);
    pos[0] = cx;
    pos[1] = cy;
}
coordinates sGetMouseDelta(void)
{
    coordinates result = {dcx,dcy};
    return result;
}
void sSetMousePosition(float x,float y)
{
    glfwSetCursorPos(s_pWindow,x,y);
    cx = x;
    cy = y;
    dcx = 0.0;
    dcy = 0.0;
}
static int8_t _kbd_states[512] = {0};
static int8_t _kbd_p_states[512] = {0};
static uint8_t _mbs[] = {0,0,0,0,0,0,0,0};
static float_t _mouse_scroll[] = {0.0,0.0};
int sMouseGetKeyState(int key)
{
    return _mbs[key];
    //return glfwGetMouseButton(s_pWindow,key);
}

float sMouseGetVerticalScroll(void)
{
	return _mouse_scroll[1];
}

float sMouseGetHorizontalScroll(void)
{
	return _mouse_scroll[0];
}

static void _poll_mouse_buttons(void)
{
    for (uint8_t button=0;button<8;button++)
    {
        _Bool action = glfwGetMouseButton(s_pWindow,button);
        
        if (_mbs[button]==0 && action)  {_mbs[button] = 1;return;}
        if (_mbs[button]==1 && action)  {_mbs[button] = 2;return;}
        if (_mbs[button]==1 && !action) {_mbs[button] = 3;return;}
        if (_mbs[button]==2 && !action) {_mbs[button] = 3;return;}
        if (_mbs[button]==3 && !action) {_mbs[button] = 0;return;}
        _mbs[button] = 2*action;
    }
}

int gflwGetMbutton(int bttn)
{
	return glfwGetMouseButton(s_pWindow, bttn);
}

static void _mouse_scroll_callback(GLFWwindow* window, double xoffset, double yoffset)
{
	_mouse_scroll[0] = xoffset;
	_mouse_scroll[1] = yoffset;
}

void S_PollCursorDelta(void)
{
    pcx = cx;
    pcy = cy;
    float x,y;
    sMouseGetPosition(&x,&y);
    cx = x;
    cy = y;
    dcx = cx-pcx;
    dcy = cy-pcy;
}

static void _keyboard_callback()
{
	for (uint32_t key = 32; key <= GLFW_KEY_LAST; key++)
	{
		if (key==163)
		{
			key = 256;
		}
		int action = glfwGetKey(s_pWindow, key);
		action = action!=0;// || action==GLFW_REPEAT;
		if (action != _kbd_p_states[key])
		{
			if (action > _kbd_p_states[key])
			{
				_kbd_states[key] = 1;
			}
			if (action < _kbd_p_states[key])
			{
				_kbd_states[key] = 3;
			}
		}
		else
		{
			_kbd_states[key] = action*2;
		}
		_kbd_p_states[key] = action;
	}
	//printf("KEY %i %hhi\n", key, _kbd_states[key]);
}

int sKeyboardGetKeyState(int key)
{
    return _kbd_states[key];
}

static uint16_t g_letters[][2] = {
        {0x82, 0x201A}, // SINGLE LOW-9 QUOTATION MARK
        {0x83, 0x0453}, // CYRILLIC SMALL LETTER GJE
        {0x84, 0x201E}, // DOUBLE LOW-9 QUOTATION MARK
        {0x85, 0x2026}, // HORIZONTAL ELLIPSIS
        {0x86, 0x2020}, // DAGGER
        {0x87, 0x2021}, // DOUBLE DAGGER
        {0x88, 0x20AC}, // EURO SIGN
        {0x89, 0x2030}, // PER MILLE SIGN
        {0x8A, 0x0409}, // CYRILLIC CAPITAL LETTER LJE
        {0x8B, 0x2039}, // SINGLE LEFT-POINTING ANGLE QUOTATION MARK
        {0x8C, 0x040A}, // CYRILLIC CAPITAL LETTER NJE
        {0x8D, 0x040C}, // CYRILLIC CAPITAL LETTER KJE
        {0x8E, 0x040B}, // CYRILLIC CAPITAL LETTER TSHE
        {0x8F, 0x040F}, // CYRILLIC CAPITAL LETTER DZHE
        {0x90, 0x0452}, // CYRILLIC SMALL LETTER DJE
        {0x91, 0x2018}, // LEFT SINGLE QUOTATION MARK
        {0x92, 0x2019}, // RIGHT SINGLE QUOTATION MARK
        {0x93, 0x201C}, // LEFT DOUBLE QUOTATION MARK
        {0x94, 0x201D}, // RIGHT DOUBLE QUOTATION MARK
        {0x95, 0x2022}, // BULLET
        {0x96, 0x2013}, // EN DASH
        {0x97, 0x2014}, // EM DASH
        {0x99, 0x2122}, // TRADE MARK SIGN
        {0x9A, 0x0459}, // CYRILLIC SMALL LETTER LJE
        {0x9B, 0x203A}, // SINGLE RIGHT-POINTING ANGLE QUOTATION MARK
        {0x9C, 0x045A}, // CYRILLIC SMALL LETTER NJE
        {0x9D, 0x045C}, // CYRILLIC SMALL LETTER KJE
        {0x9E, 0x045B}, // CYRILLIC SMALL LETTER TSHE
        {0x9F, 0x045F}, // CYRILLIC SMALL LETTER DZHE
        {0xA0, 0x00A0}, // NO-BREAK SPACE
        {0xA1, 0x040E}, // CYRILLIC CAPITAL LETTER SHORT U
        {0xA2, 0x045E}, // CYRILLIC SMALL LETTER SHORT U
        {0xA3, 0x0408}, // CYRILLIC CAPITAL LETTER JE
        {0xA4, 0x00A4}, // CURRENCY SIGN
        {0xA5, 0x0490}, // CYRILLIC CAPITAL LETTER GHE WITH UPTURN
        {0xA6, 0x00A6}, // BROKEN BAR
        {0xA7, 0x00A7}, // SECTION SIGN
        {0xA8, 0x0401}, // CYRILLIC CAPITAL LETTER IO
        {0xA9, 0x00A9}, // COPYRIGHT SIGN
        {0xAA, 0x0404}, // CYRILLIC CAPITAL LETTER UKRAINIAN IE
        {0xAB, 0x00AB}, // LEFT-POINTING DOUBLE ANGLE QUOTATION MARK
        {0xAC, 0x00AC}, // NOT SIGN
        {0xAD, 0x00AD}, // SOFT HYPHEN
        {0xAE, 0x00AE}, // REGISTERED SIGN
        {0xAF, 0x0407}, // CYRILLIC CAPITAL LETTER YI
        {0xB0, 0x00B0}, // DEGREE SIGN
        {0xB1, 0x00B1}, // PLUS-MINUS SIGN
        {0xB2, 0x0406}, // CYRILLIC CAPITAL LETTER BYELORUSSIAN-UKRAINIAN I
        {0xB3, 0x0456}, // CYRILLIC SMALL LETTER BYELORUSSIAN-UKRAINIAN I
        {0xB4, 0x0491}, // CYRILLIC SMALL LETTER GHE WITH UPTURN
        {0xB5, 0x00B5}, // MICRO SIGN
        {0xB6, 0x00B6}, // PILCROW SIGN
        {0xB7, 0x00B7}, // MIDDLE DOT
        {0xB8, 0x0451}, // CYRILLIC SMALL LETTER IO
        {0xB9, 0x2116}, // NUMERO SIGN
        {0xBA, 0x0454}, // CYRILLIC SMALL LETTER UKRAINIAN IE
        {0xBB, 0x00BB}, // RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK
        {0xBC, 0x0458}, // CYRILLIC SMALL LETTER JE
        {0xBD, 0x0405}, // CYRILLIC CAPITAL LETTER DZE
        {0xBE, 0x0455}, // CYRILLIC SMALL LETTER DZE
        {0xBF, 0x0457}  // CYRILLIC SMALL LETTER YI
};

void sStringUnicodeTo1251(char* result, char* string)
{
	int length = strlen(string);
	int i=0,i1251=0;
	for (; i<length; i++, i1251++)
	{
		_Bool found = 0;
		uint16_t chr = *(uint16_t*)(string+i);
		chr = ((chr&0xFF)<<8) | (chr>>8);
		for (int c=0; c<sizeof(g_letters)/sizeof(g_letters[0]); c++)
		{
			if (chr==g_letters[c][1])
			{
				result[i1251] = g_letters[c][0];
				i++;
				found = 1;
				break;
			}
		}
		if (!found)
		{
			result[i1251] = string[i];
		}
	}
	result[i1251] = 0;
}

//Window
void error_callback(int error, const char* description)
{
    fprintf(stderr, "Error: %s\n", description);
}

void sEngineSetSwapInterval(uint32_t interval)
{
    glfwSwapInterval(interval);
}

void sEngineCreateWindow(uint16_t width,uint16_t height,_Bool fullscreen)
{
    glfwInit();
    glfwSetErrorCallback(error_callback);
    GLFWmonitor* pMonitor = fullscreen ? glfwGetPrimaryMonitor() : NULL;
    
    if (width==0 && height==0)
    {
        const GLFWvidmode * mode = glfwGetVideoMode(pMonitor);
        s_pWindow = glfwCreateWindow(mode->width, mode->height, "SGM OpenGL", pMonitor, NULL);
    }
    else
    {
        s_pWindow = glfwCreateWindow(width, height, "SGM OpenGL", pMonitor, NULL);
    }
    
    if (!s_pWindow) GLFW_START_ERROR;
    glfwMakeContextCurrent(s_pWindow);
    GLenum res = glewInit();
    if (res!=GLEW_OK)
    {
        fprintf(stderr,"%s",glewGetErrorString(res));
        GL_START_ERROR;
    }
    glfwSwapInterval(1);
    const GLubyte* vendor = glGetString(GL_VENDOR);
    const GLubyte* renderer = glGetString(GL_RENDERER);
    printf("Video system:\nVendor %s\nRender %s\n",vendor,renderer);
    glfwSetScrollCallback(s_pWindow,_mouse_scroll_callback);
    glfwSetInputMode(s_pWindow, GLFW_CURSOR, GLFW_CURSOR_HIDDEN);
    //glfwSetCharCallback(s_pWindow, _entry_callback);
    //glfwSetKeyCallback(s_pWindow, _entry_special_callback);
    //glfwSetKeyCallback(s_pWindow, _keyboard_callback);
    //glfwSetMouseButtonCallback(s_pWindow,mouse_button_callback);
}

int sEngineGetWidth(void)
{
	int width;
	glfwGetFramebufferSize(s_pWindow, &width, (int*)0);
	return width;
}

int sEngineGetHeight(void)
{
	int height;
	glfwGetFramebufferSize(s_pWindow, (int*)0, &height);
	return height;
}

void sMouseHide(void)
{
    glfwSetInputMode(s_pWindow, GLFW_CURSOR, GLFW_CURSOR_HIDDEN);
}

void sMouseShow(void)
{
    glfwSetInputMode(s_pWindow, GLFW_CURSOR, GLFW_CURSOR_NORMAL);
}


void sEngineStartOpenGL(void)
{
    glc(glDisable(GL_MULTISAMPLE));
    glc(glCullFace(GL_BACK));
    glc(glClearColor(0.75,0.8,1.0,0.0));
    glc(glClearDepth(1.0));
    glc(glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT));
    glc(glDepthFunc(GL_LEQUAL));
    glc(glEnable(GL_DEPTH_TEST));
    glc(glEnable(GL_CULL_FACE));
    //glDisable(GL_CULL_FACE);
    
    glc(glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA));
    glc(glEnable(GL_BLEND));
    glc(glEnable(GL_ALPHA_TEST));
    glstarted = 1;
    
    sRenderLoadShaders();

    if (_renderRayTracing)
    {
    	noise_texture.height = noise_texture.width = 8;
		sTextureGenerateBlueNoise(&noise_texture);
    }

    formsInit();
}

void execute_objects(sScene* scene)
{
    for (index_t i=0;i<scene->gobjects_count;i++)
    {
        sObject* obj = (sObject*)scene->gobjects[i];
        if (obj->behaviour)
        {
            obj->behaviour(obj);
        }
        if (!active_scene)
        {
        	return;
        }
    }
}

double sGetFrameTime(void)
{
    return sFrameTime;
}

_Bool sGetInputTick(void)
{
    if (frame_input_time>=0.016666)
    {
        frame_input_time = 0.0;
        return 1;
    }
    return 0;
}

/*static void auto_disable(sScene* scene)
 * {
 *        for (uint32_t i=0;i<scene->gobjects_count;i++)
 *        {
 *                sObject* obj = (sObject*)scene->gobjects[i];
 *                if (obj->physicsType<2) continue;
 *                const dReal* vel = dBodyGetLinearVel(obj->body);
 *                obj->averangeVel = obj->averangeVel*0.5 + vel[0]*vel[0] + vel[1]*vel[1] + vel[2]*vel[2];
 *                if (obj->averangeVel<0.02 && obj->averangeVel>0.001)
 *                {
 *                        dBodyDisable(obj->body);
 *                }
 *        }
 * }*/

char *sGetProfilingString(void)
{
	sprintf(sDebugString,"Averange FPS %.2lf (%.6lf ms)\n"
						 "Logic %.3lf%% (%.6lf ms)\n"
						 "Physics %.3lf%% (%.6lf ms)\n"
						 "Render %.3lf%% (%.6lf ms)\n",
						 1.0/sFrameAverangeTime, sFrameAverangeTime,
						 sLogicAverangeTime/sFrameAverangeTime*100.0, sLogicAverangeTime,
						 sPhysicsAverangeTime/sFrameAverangeTime*100.0, sPhysicsAverangeTime,
						 sRenderAverangeTime/sFrameAverangeTime*100.0, sRenderAverangeTime );
	return sDebugString;
}

void io_thread(void)
{
	memset(_mouse_scroll,0,sizeof(_mouse_scroll));
    glfwPollEvents();
    _poll_mouse_buttons();
    _keyboard_callback();
}

void *logic_thread(void *ptr)
{
	sScene* scene = ptr;
	io_thread();
	if (!scene)
	{
		return 0;
	}

    // Physics //
    sProfilingTimers.physics = glfwGetTime();
    if (sGetFrameTime()>0.025)
    {
        sPhysicsStep(scene,0.016666);
        sPhysicsStep(scene,0.016666);
    }
    else
    {
    	sPhysicsStep(scene,0.016666);
    }
    sProfilingTimers.physics = glfwGetTime() - sProfilingTimers.physics;
    /////////////

    // Logic //
	sProfilingTimers.animations = glfwGetTime();
	for (index_t i=0;i<scene->skeletons_count;i++)
	{
		sActionProcess(scene->skeletons[i]);
	}
	sProfilingTimers.animations = glfwGetTime() - sProfilingTimers.animations;

	sProfilingTimers.scripts = glfwGetTime();
    sSceneFunctionLoop(scene);
    execute_objects(scene);
    if (scene->behaviour)
    	scene->behaviour(scene);
    if (active_scene==0)
    {
    	puts("Scene was deleted\n");
    	return (void*)1;
    }
    sProfilingTimers.scripts = glfwGetTime() - sProfilingTimers.scripts;
    ///////////
    S_PollCursorDelta();
    return 0;
}

void sEngineStartLoop(void)
{
    double ftimer = glfwGetTime();

    printf("SGM started (%llu bytes)\n",sGetAllocatedMem());
    uint32_t frame = 0;
    while (!glfwWindowShouldClose(s_pWindow))
    {
    	if (_renderApplyChanges)
    	{
    		puts("renderApplyChanges");
    		sRenderDestroyShaders();
    		sRenderLoadShaders();
    		if (active_scene)
    		{
    			sCameraDestroyFB(&active_scene->camera);
    			sCameraInitFB(&active_scene->camera);
    		}
			_renderApplyChanges = 0;
    	}

    	sTime = glfwGetTime();
        ftimer = sTime;
#ifdef THREADS
    	pthread_t thread;
#else
    	if (logic_thread(active_scene))
		{
    		continue;
		}
#endif

    	// Placing objects //
    	sProfilingTimers.placing = glfwGetTime();
    	if (active_scene)
    	{
			for (index_t i=0;i<active_scene->gobjects_count;i++)
			{
				if (((sObject*)active_scene->gobjects[i])->parent) continue;
				sObjectPlaceChildren((sObject*)active_scene->gobjects[i]);
			}
	        sSoundPlaceSources();
    	}
        sProfilingTimers.placing = glfwGetTime() - sProfilingTimers.placing;

#ifdef THREADS
        pthread_create(&thread, NULL, logic_thread, active_scene);
#endif

    	sRenderClear(0.25, 0.25, 0.25, 0.0);
        if (active_scene)
        {
        	sRenderDrawScene(active_scene);
        }
#ifdef THREADS
    	pthread_join(thread, NULL);
#endif

		glFlush();
		
        double kek = glfwGetTime();
		//glFinish();
		sProfilingTimers.interface = glfwGetTime() - kek;

		sProfilingTimers.buffers = glfwGetTime();
        fFormsProcess();
    	glfwSwapBuffers(s_pWindow);
    	sProfilingTimers.buffers = glfwGetTime() - sProfilingTimers.buffers;
        sFrameTime = glfwGetTime() - ftimer;
        frame++;
        if (glfwGetTime()>=1.0)
        {
        	printf("FPS %u\n",frame);
        	frame = 0;
        	glfwSetTime(0.0);
        }
        if (sKeyboardGetKeyState(GLFW_KEY_ESCAPE)==GLFW_PRESS) break;
    }
    sEngineClose();
}

void sEngineSwapBuffers(void)
{
	glfwSwapBuffers(s_pWindow);
}

int sEngineShouldClose(void)
{
	return glfwWindowShouldClose(s_pWindow);
}

void sEngineClose(void)
{
    fFormsClear();
    sSceneFree(active_scene);
    glfwDestroyWindow(s_pWindow);
    glfwTerminate();
    printf("SGM stopped (%llub)\n",sGetAllocatedMem());
}


void* sCalloc(size_s size,size_s count)
{
    size_s block = size*count+8;
    bytes_allocated += block;
    void* ptr = calloc(block,1);
    *(size_s*)ptr = block;
    return ptr+8;
}

void* sMalloc(size_s size)
{
    size+=8;
    bytes_allocated += size;
    void* ptr = malloc(size);
    *(size_s*)ptr = size;
    return ptr+8;
}

void* sRealloc(void* old_ptr,size_s size)
{
    if (!old_ptr)
    {
        return sMalloc(size);
    }
    size+=8;
    void* new_ptr;
    bytes_allocated += size - *(size_s*)(old_ptr-8);
    new_ptr = realloc(old_ptr-8,size);
    *(size_s*)new_ptr = size;
    return new_ptr+8;
}

void* sRecalloc(void* old_ptr,size_s size)
{
    if (!old_ptr)
    {
        return sCalloc(size,1);
    }
    uint64_t previous_size = sSizeof(old_ptr);
    size+=8;
    void* new_ptr;
    bytes_allocated += size - previous_size;
    new_ptr = realloc(old_ptr-8,size);
    *(size_s*)new_ptr = size;
    memset(new_ptr+previous_size,0,size-previous_size);
    return new_ptr+8;
}

void sFree(void* ptr)
{
	if (!ptr)
	{
		return;
	}
    bytes_allocated -= *(size_s*)(ptr-8);
    free(ptr-8);
}

size_s sSizeof(void* ptr)
{
	return *(size_s*)(ptr-8);
}

size_s sGetAllocatedMem(void)
{
    return bytes_allocated;
}

