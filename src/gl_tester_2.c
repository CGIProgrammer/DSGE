#include <SDL2/SDL.h>
#include "structures/types.h"
#include "structures/shader.h"
#include "structures/components/light_component.h"
#include "time.h"
#include "io.h"

SDL_Window *window;
int screen_width = 1920; // ширина окна
int screen_height = 1080; // высота окна
bool running = true;

int stbi_write_jpg(char const *filename, int x, int y, int comp, const void *data, int quality);
void screenshot(void)
{
    struct tm * timeinfo;
    time_t rawtime;
    time (&rawtime);
    timeinfo = localtime (&rawtime);

    glc(glBindFramebuffer(GL_FRAMEBUFFER, 0));
    char (*data)[screen_width][3] = (char(*)[screen_width][3])malloc((screen_height+1)*screen_width*3);
    glReadPixels(0, 0, screen_width, screen_height, GL_RGB, GL_UNSIGNED_BYTE, data);
    for (int i=0; i<screen_height/2; i++) {
        void *buffer = data[screen_height];
        memcpy(buffer, data[i], screen_width*3);
        memcpy(data[i], data[screen_height-i-1], screen_width*3);
        memcpy(data[screen_height-i-1], buffer, screen_width*3);
    }
    char fname[256];
    sprintf(fname, "screenshots/%s.jpg", asctime(timeinfo));
    stbi_write_jpg(fname, screen_width, screen_height, 3, data, 100);
    free(data);
}

int pmx, pmy;
int pbtn;
laMatrix center = {{1.0,0.0,0.0}, 4};
float dist = 10.0;
float angle_x = 63.6, angle_z = 46.7-90.0;

sGameObjectID* draw_list = 0;
sGameObjectID* lights_list = 0;
sGameObjectID camera;
sGameObjectID object;
sGameObjectID spotlight;
sMeshID monkey;
sMaterialID material;
sTextureID texture;
sTextureID normalmap;
sTextureID cubemap;
sMaterialID plane_material;

int buttons[SDL_NUM_SCANCODES];
int buttons_delta[SDL_NUM_SCANCODES];

void mouse_look(bool fr)
{
    int dx, dy;
    int btn = SDL_GetRelativeMouseState(&dx, &dy);
    
    int numkeys;
    const uint8_t* keyboard = SDL_GetKeyboardState(&numkeys);
    if (btn!=pbtn) {
        SDL_GetMouseState(&pmx, &pmy);
        pbtn = btn;
        return;
    }
    if (btn==2 || fr) {
        if (keyboard[SDL_SCANCODE_LSHIFT]) {
            laMatrix xdir = laMulf(laMatrixGetXDirection(camera->transform.global), -dx*0.0004*dist);
            laMatrix ydir = laMulf(laMatrixGetYDirection(camera->transform.global),  dy*0.0004*dist);
            center = laAdd(center, laAdd(xdir, ydir));
        } else {
            angle_x -= dy * 0.15;
            angle_z -= dx * 0.15;
            camera->transform.global = laRotationXYZ(radians(angle_x), 0.0f, radians(angle_z));
        }
        camera->transform.global.a[3]  = center.a[0] + camera->transform.global.a[ 2] * dist;
        camera->transform.global.a[7]  = center.a[1] + camera->transform.global.a[ 6] * dist;
        camera->transform.global.a[11] = center.a[2] + camera->transform.global.a[10] * dist;
        SDL_GetMouseState(&pmx, &pmy);
        pmx = pmx + (screen_width-1) * ((pmx<=0) - (pmx>=(screen_width-1)));
        pmy = pmy + (screen_height-1) * ((pmy<=0) - (pmy>=(screen_height-1)));
        SDL_WarpMouseInWindow(window, pmx, pmy);
    }
    pbtn = btn;
    for (int i=0; i<SDL_NUM_SCANCODES; i++)
    {
        if (keyboard[i])
        {
            if (i==SDL_SCANCODE_LEFT  && plane_material->roughness>0.01)
            {
                plane_material->roughness -= 0.01;
            }
            if (i==SDL_SCANCODE_RIGHT && plane_material->roughness<0.99)
            {
                plane_material->roughness += 0.01;
            }
            if (i==SDL_SCANCODE_UP  && dist>0.5)
            {
                dist -= 0.03;
            }
            if (i==SDL_SCANCODE_DOWN)
            {
                dist += 0.03;
            }
        }
        buttons[i] = keyboard[i];
    }
    fflush(stdout);
}

int main(/*int argc, char *argv[]*/)
{
    sSceneID scene;
    if (SDL_Init(SDL_INIT_VIDEO))
    {
        printf("SDL2 initialization failed: %s\n", SDL_GetError());
        return 1;
    }

    //int r,g,b,a;
    SDL_GL_SetAttribute(SDL_GL_RED_SIZE, 8);
    SDL_GL_SetAttribute(SDL_GL_GREEN_SIZE, 8);
    SDL_GL_SetAttribute(SDL_GL_BLUE_SIZE, 8);
    SDL_GL_SetAttribute(SDL_GL_ALPHA_SIZE, 8);
    SDL_GL_SetAttribute(SDL_GL_DEPTH_SIZE, 24);
    SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);
    //SDL_GL_SetAttribute(SDL_GL_MULTISAMPLEBUFFERS, 1);
    //SDL_GL_SetAttribute(SDL_GL_MULTISAMPLESAMPLES, 8);


    window = SDL_CreateWindow((char*)"Cube", SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED, screen_width, screen_height, SDL_WINDOW_SHOWN | SDL_WINDOW_OPENGL);
    SDL_GLContext glcontext = SDL_GL_CreateContext(window);
    if(window == NULL || glcontext == NULL)
    {
        printf("window %p\n", window);
        printf("glcontext %p\n", glcontext);
        return 1;
    }

    SDL_GL_SetSwapInterval(0);

    #ifdef GLAD
    gladLoadGL();
    #endif
    #ifdef GLEW
    glewInit();
    #endif

    sShaderSetVersion((const char*)glGetString(GL_SHADING_LANGUAGE_VERSION));
    printf("Mem %lu\n", sGetAllocatedMem());
    //create_simple_scene();
    glc(glDisable(GL_DEPTH_TEST));
    glc(glDisable(GL_CULL_FACE));
    glc(glEnable(GL_MULTISAMPLE));
    glc(glEnable(GL_BLEND));
    //float a = 0.0;

    scene = sSceneLoadBin("data/scenes/shooting_range.scene");
    puts("Create sky");
    sTextureID sky = sTextureCreateCubemap("Skybox", 2048, 2048, RGB16F, 1, 1);
    puts("Spliting cubemap");
    sTextureCubeSplit(sky);
    puts("Baking skybox");
    sCameraComponentBakeSkybox(sky);
    puts("Generating mipmaps");
    sTextureGenerateMipMaps(sky);
    puts("Setting skybox");
    //sky = sTextureLoadDDSCubemap("/home/ivan/SGM_SDK/SGE/data/textures/cubemap/kitchen_shup.dds");
    sSceneSetSkybox(scene, sky);
    camera = scene->camera;
    
    //sGameObjectID spotlight = sGameObjectCreate((char*)"spotlight");
    //spotlight->transform.global = laRotationXYZ(radians(65.261), radians(3.16371), radians(102.0));
    //spotlight->transform.global.a[3]  = -0.727522;
    //spotlight->transform.global.a[7]  = -17.492493;
    //spotlight->transform.global.a[11] = 3.417613;
    //spotlight->light_component = sLightCreateShadowBuffer(2048, sLightPoint, 1);
    //spotlight->light_component->user = spotlight;
    //spotlight->light_component->color = (sColor){
    //100.0,
    //100.0,
    //100.0,1.0};
    //sSceneAddObject(scene, spotlight);

    //scene = sSceneMakeDemo();
    //puts("Creating camera");
    //puts("Render initialization");
    //sSceneSetSkybox(scene, sTextureLoadDDSCubemap("/home/ivan/SGM_SDK/SGE/data/textures/cubemap/cloudySea.dds"));
    //camera = sGameObjectCreate("camera");
    //camera->camera_component = sCameraInitDeferredRenderer(width, height, 80.0, 1);
    //camera->camera_component->user = camera;
    //sSceneSetActiveCamera(scene, camera);

    //camera->transform.global = laRotationXYZ(radians(63.6), 0.0f, radians(46.7));
    //camera->transform.global.a[3]  = 7.35889;
    //camera->transform.global.a[7]  =-6.50764;
    //camera->transform.global.a[11] = 4.95831 + 1;
    //puts("Setting camera");
    //mouse_look(1);

    while (running){
        SDL_Event event; // события SDL

		while ( SDL_PollEvent(&event) ){ // начинаем обработку событий
			switch(event.type){ // смотрим:
            case SDL_QUIT: // если произошло событие закрытия окна, то завершаем работу программы
                running = false;
                break;

            case SDL_KEYDOWN: // если нажата клавиша
                switch(event.key.keysym.sym){ // смотрим какая
                    case SDLK_ESCAPE: // клавиша ESC
                        running = false; // завершаем работу программы
                        break;
                    case SDLK_F12: // Клавиша F12
                        screenshot();
                        break;
                }
                break;
			}
		}
        mouse_look(0);
        //object->transform.global = laRotationXYZ(0.0, 0.0, a);
        //a += 0.01f;
        //object->transform.global.a[11] += 1.0;
        //draw();
        sSceneDraw(scene);
        glFlush();
        //screenshot("scrn.jpg");
		SDL_GL_SwapWindow(window);
        //running = 0;
    }
    sSceneDelete(scene);
    sGameObjectClear();
    sMeshClear();
    sMaterialClear();
    sShaderClear();
    sTextureClear();
    sShaderDeleteDict();
    
    sDelete(draw_list);
    sDelete(lights_list);

    printf("Mem %lu\n", sGetAllocatedMem());
    //puts(glGetString(GL_SHADING_LANGUAGE_VERSION));
    SDL_Quit();

    laPrint(laPerspective(1024, 1024, 0.1, 100.0, 90.0));
    laPrint(laInverted(laPerspective(1024, 1024, 0.1, 100.0, 90.0)));

    return 0;
}
