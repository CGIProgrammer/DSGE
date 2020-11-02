/*
 * shader.c
 *
 *  Created on: 22 дек. 2017 г.
 *      Author: ivan
 */

#include "structures/shader.h"
#include "io.h"

#ifndef MAX_INSTANCES
#define MAX_INSTANCES 100
#endif

#ifdef __cplusplus
extern "C" {
#endif

#define sShaderCacheUniform(shader, vars, uniform) {shader->vars[uniform] = sShaderGetUniformLocation(shader, #uniform);}

sShaderID sShaderActive = 0;
static sShaderID* shaders = 0;
static sDict shaders_ids = {0, 0};

const char* vertex_head = 
"#if defined(GL_ARB_explicit_attrib_location) && __VERSION__ < 130 || __VERSION__ > 120\n"
	"layout (location=0) in vec3 pos;\n"
	"layout (location=1) in vec3 nor;\n"
	"layout (location=2) in vec2 uv;\n"
	"layout (location=3) in vec3 bin;\n"
	"layout (location=4) in vec3 tang;\n"
	"layout (location=5) in vec2 uv2;\n"
	"layout (location=6) in vec3 weights;\n"
"#else\n"
	"attribute vec3 pos;\n"
	"attribute vec3 nor;\n"
	"attribute vec2 uv;\n"
	"attribute vec3 bin;\n"
	"attribute vec3 tang;\n"
	"attribute vec2 uv2;\n"
	"attribute vec3 weights;\n"
"#endif\n"
"#define VERTEX\n";

const char* fragment_head = 
"#if __VERSION__ == 100 || __VERSION__ == 120\n"
"  #define fragColor gl_FragColor\n"
"  #define fragData gl_FragData\n"
"#else\n"
"  output vec4 fragColor;\n"
"  output vec4 fragData[6];\n"
"#endif\n"
"#define FRAGMENT\n"
;

const char* cross_version_head = 
"#extension GL_ARB_explicit_attrib_location : enable\n"
"#extension GL_ARB_gpu_shader5 : enable\n"
"#extension GL_EXT_shader_texture_lod : enable\n"
"#extension GL_EXT_texture_array : enable\n"
"#if __VERSION__ > 120\n"
"  #define input in\n"
"  #define output out\n"
"#else\n"
"  #define input varying\n"
"  #define output varying\n"
"#endif\n";

static char GLSL_VERSION[16] = "330";
static char precision[64] = "precision mediump float;\n";


void sShaderSetVersion(const char* version)
{
	printf("Setting GLSL version %s\n", version);
	for (int i=0,j=0; version[i]; i++) {
		if (version[i]=='.') continue;
		GLSL_VERSION[j] = version[i];
		j++;
	}
}

char shader_log_buffer[100000];

static size_t _size_of_file(FILE* fp)
{
	size_t sz;
	fseek(fp, 0L, SEEK_END);
	sz = ftell(fp);
	fseek(fp, 0L, SEEK_SET);
	return sz;
}

static char* _load_file(const char* file_name)
{
	FILE* fp = fopen(file_name, "rb");if (!fp)
	{
		fprintf(stderr,"%s not such file\n", file_name);
		exit(-1);
	}
	size_t fsize = _size_of_file(fp);
	char* source = sNewArray(char, fsize);
	readf((void*)source, 1, fsize, fp);
	fclose(fp);
	listPushBack((void**)&source, (void*)"\n", 1);
	return source;
}

bool _renderRayTracing = 1;
static char cmd[10000];

char* _include_linker(char *source, int vertex, char* defines)
{
	char file_name[256];
	char* linked = 0;
	sprintf(cmd, "#version %s\n"
				 "#define MAX_INSTANCES %d\n"
				 "#define TEXTURE_RANDOM 1\n"
				 "#define MAX_LIGHTS %d\n",
				 GLSL_VERSION,
				 MAX_INSTANCES,
				 MAX_LIGHTS
				 );

	listPushBack((void**)&linked, (void*)cmd, strlen(cmd));
	listPushBack((void**)&linked, (void*)"#define _SSGI\n", 14);
	listPushBack((void**)&linked, (void*)"#define _SSR \n", 14);
	listPushBack((void**)&linked, (void*)cross_version_head, strlen(cross_version_head));
	if (strstr(GLSL_VERSION, "300") || 
		strstr(GLSL_VERSION, "310") || 
		strstr(GLSL_VERSION, "320") || 
		strstr(GLSL_VERSION, "ES") || 
		strstr(GLSL_VERSION, "es"))
	{
		listPushBack((void**)&linked, (void*)precision, strlen(precision));
	}
	if (vertex) {
		listPushBack((void**)&linked, (void*)vertex_head, strlen(vertex_head));
	} else {
		listPushBack((void**)&linked, (void*)fragment_head, strlen(fragment_head));
	}

	if (defines) {
		listPushBack((void**)&linked, (void*)defines, strlen(defines));
	}

	char* inclusion = 0;

	int lin=0;
	for (
		char *line=source, *line_end=strchr(source, '\n')+1;
		(uintptr_t)line >= (uintptr_t)source && (uintptr_t)line < (uintptr_t)line_end;
		line=line_end, line_end=strchr(line, '\n')+1
	) {
		lin++;
		if (strncmp(line, "//", 2)) {
			char* inc_start = strstr(line, "#include");
			char* file_name_start;
			char* file_name_end;
			if (inc_start && (uintptr_t)inc_start<(uintptr_t)line_end) {
				for (file_name_start=inc_start; *file_name_start!='"'; file_name_start++);
				file_name_start++;
				for (file_name_end=file_name_start; *file_name_end!='"'; file_name_end++);
				uintptr_t len = (uintptr_t)file_name_end - (uintptr_t)file_name_start;
				memset(file_name, 0, sizeof(file_name));
				strncpy(file_name, file_name_start, len);
				inclusion = _load_file(file_name);
				listPushBack((void**)&linked, inclusion, sSizeof(inclusion));
				sDelete(inclusion);
			} else {
				listPushBack((void**)&linked, line, (uintptr_t)line_end - (uintptr_t)line);
			}
		} else {
			listPushBack((void**)&linked, line, (uintptr_t)line_end - (uintptr_t)line);
		}
	}
	listPushBack((void**)&linked, (void*)"\n", 2);
	return linked;
}

static void _print_lines(char* text)
{
	unsigned line=1;
    fprintf(stderr, "%04d ",line++);
	for (; *text; text++)
	{
		fprintf(stderr, "%c", *text);
		if (*text=='\n')
		{
			fprintf(stderr, "%04d ",line++);
		}
	}
	fputs("", stderr);
}

static inline intptr_t sShaderGetUniformLocation(sShaderID shader, char* name)
{
	return (intptr_t)glGetUniformLocation(shader->program_id, name);
	intptr_t uniform = (intptr_t)sDictGetItemKW(&shader->uniform_cache, name);
	if (uniform)
	{
		uniform--;
	}
	else
	{
    	uniform = (intptr_t)glGetUniformLocation(shader->program_id, name);
		if (uniform==-1) return -1;
		sDictAddItemKW(&shader->uniform_cache, name, (void*)(uniform+1));
	}
	return uniform;
}

static void sShaderMake(sShaderID shader)
{
	shader->program_id = glCreateProgram();

	glBindAttribLocation(shader->program_id, 0, "pos");
	glBindAttribLocation(shader->program_id, 1, "nor");
	glBindAttribLocation(shader->program_id, 2, "uv");

	glAttachShader(shader->program_id,shader->vertex_id);
	glAttachShader(shader->program_id,shader->fragment_id);
	glLinkProgram(shader->program_id);
	glGetProgramiv(shader->program_id, GL_LINK_STATUS, &shader->success);
	if (!shader->success)
	{
		puts("Shader make error");
	}
	glGetProgramInfoLog(shader->program_id, sizeof(shader_log_buffer), &shader->log_len, shader_log_buffer);
	if (!shader->success)
	{
		if (!strncmp(shader_log_buffer, "Fragment", 8))
		{
			_print_lines(shader->fragment_source);
		}
		if (!strncmp(shader_log_buffer, "Vertex", 8))
		{
			_print_lines(shader->vertex_source);
		}
		printf("Link error: \n%s\n",shader_log_buffer);
		exit(-1);
		return;
	}
	else
	{
		//puts(shader_log_buffer);
	}
}

static sShaderID sShaderCreate(void)
{
	sShaderID shader = sNew(sShader);
	sListPushBack(shaders, shader);
	return shader;
}

static sShaderID sShaderFindByIDs(GLuint vid, GLuint fid)
{
	size_t shdcnt = sListGetSize(shaders);
	for (size_t i=0; i<shdcnt; i++)
	{
		if (vid==shaders[i]->vertex_id && fid==shaders[i]->fragment_id)
		{
			return shaders[i];
		}
	}
	return (sShaderID)0;
}

static GLuint sShaderLoadFromFile(const char* name, const char* file_name, bool fragment, const char* defines)
{
	printf("Loading shader named \"%s\"\n", name);
	uintptr_t shid = (uintptr_t)sDictGetItemKW(&shaders_ids, (char*)name);
	GLsizei success = 1, log_length = 0;
	if (shid)
	{
		//printf("\"%s\" already compiled. Just return its ID (%lu)\n", name, shid-1);
		return shid-1;
	}
	char* source = _load_file(file_name);
	char* preprocessed = _include_linker(source, !fragment, (char*)defines);
	sDelete(source);

	shid = glCreateShader(fragment ? GL_FRAGMENT_SHADER : GL_VERTEX_SHADER);
	glShaderSource(shid, 1, (const GLchar *const*)&preprocessed, 0);
	glCompileShader(shid);

	//GLint link;
	glGetShaderiv(shid, GL_COMPILE_STATUS, &success);
	if (!success)
	{
		fputs("Shader error", stderr);
	}
	glGetShaderInfoLog(shid, sizeof(shader_log_buffer), &log_length, shader_log_buffer);
	if (!success)
	{
		_print_lines(preprocessed);
		fprintf(stderr, "%s shader %s has error:\n%s\n", fragment ? "Fragment" : "Vertex", name, shader_log_buffer);
		exit(-1);
	}

	//printf("\"%s\" is not compiled. Created new id (%lu)\n", name, shid);
	sDictAddItemKW(&shaders_ids, (char*)name, (void*)(shid+1));
	sDelete(preprocessed);
	return shid;
}

static void sShaderCacheUniforms(sShaderID shader)
{
	sShaderCacheUniform(shader, base_vars, vObjectTransform);
	sShaderCacheUniform(shader, base_vars, vCameraTransform);
	sShaderCacheUniform(shader, base_vars, vCameraProjection);
	sShaderCacheUniform(shader, base_vars, vCameraProjectionInv);
	sShaderCacheUniform(shader, base_vars, lSpotCount);
	sShaderCacheUniform(shader, base_vars, lPointCount);
	sShaderCacheUniform(shader, base_vars, fDistanceFormat);
	sShaderCacheUniform(shader, base_vars, fDiffuseMap);
	sShaderCacheUniform(shader, base_vars, fSpecularMap);
	sShaderCacheUniform(shader, base_vars, fReliefMap);
	sShaderCacheUniform(shader, base_vars, fLightMap);
	sShaderCacheUniform(shader, base_vars, fMetallicMap);
	sShaderCacheUniform(shader, base_vars, fRoughnessMap);
	sShaderCacheUniform(shader, base_vars, fDiffuseValue);
	sShaderCacheUniform(shader, base_vars, fSpecularValue);
	sShaderCacheUniform(shader, base_vars, fReliefValue);
	sShaderCacheUniform(shader, base_vars, fMetallicValue);
	sShaderCacheUniform(shader, base_vars, fRoughnessValue);
	sShaderCacheUniform(shader, base_vars, fFresnelValue);
	sShaderCacheUniform(shader, base_vars, fTexScrollX);
	sShaderCacheUniform(shader, base_vars, fTexScrollY);

	sShaderCacheUniform(shader, sunlight_vars, lSunColor);
	sShaderCacheUniform(shader, sunlight_vars, lSunTransform);
	sShaderCacheUniform(shader, sunlight_vars, lSunDepth);
	sShaderCacheUniform(shader, sunlight_vars, lSunShadowMap);

	char var_loc[256];

	shader->sunlight_vars[lSunColor] = sShaderGetUniformLocation(shader, "lSun.color");
	shader->sunlight_vars[lSunTransform] = sShaderGetUniformLocation(shader, "lSun.itransform");
	shader->sunlight_vars[lSunDepth] = sShaderGetUniformLocation(shader, "lSun.depth_range");
	shader->sunlight_vars[lSunShadowMap] = sShaderGetUniformLocation(shader, "lSunShadowMap");

	for (int i=0; i<MAX_LIGHTS; i++)
	{
		sprintf(var_loc, "lSpots[%d].color", i); 		shader->spotlight_vars[i][lSpotColor]		= sShaderGetUniformLocation(shader, var_loc);
		sprintf(var_loc, "lSpots[%d].itransform", i); 	shader->spotlight_vars[i][lSpotTransform]	= sShaderGetUniformLocation(shader, var_loc);
		sprintf(var_loc, "lSpots[%d].blending", i); 	shader->spotlight_vars[i][lSpotBlending]	= sShaderGetUniformLocation(shader, var_loc);
		sprintf(var_loc, "lSpots[%d].angle_tan", i); 	shader->spotlight_vars[i][lSpotAngle]		= sShaderGetUniformLocation(shader, var_loc);
		sprintf(var_loc, "lSpotShadowMaps[%d]", i); 	shader->spotlight_vars[i][lSpotShadowmap]	= sShaderGetUniformLocation(shader, var_loc);
		sprintf(var_loc, "lSpots[%d].position", i); 	shader->spotlight_vars[i][lSpotPosition]	= sShaderGetUniformLocation(shader, var_loc);
	}
	for (int i=0; i<MAX_LIGHTS; i++)
	{
		sprintf(var_loc, "lPoints[%d].color", i);    shader->point_light_vars[i][lPointColor]     = sShaderGetUniformLocation(shader, var_loc);
		sprintf(var_loc, "lPoints[%d].position", i); shader->point_light_vars[i][lPointPosition]  = sShaderGetUniformLocation(shader, var_loc);
		sprintf(var_loc, "lPointShadowMaps[%d]", i); shader->point_light_vars[i][lPointShadowmap] = sShaderGetUniformLocation(shader, var_loc);
	}
}

static sShaderID sShaderLoadAndMake(
	const char* vert_name, const char* frag_name,
	const char* vert_fname, const char* frag_fname,
	const char* vert_defines, const char* frag_defines
)
{
	GLuint vid = sShaderLoadFromFile(vert_name, vert_fname, 0, vert_defines);
	GLuint fid = sShaderLoadFromFile(frag_name, frag_fname, 1, frag_defines);

	sShaderID shader = sShaderFindByIDs(vid, fid);
	if (shader) return shader;
	printf("Making \"%s\" \"%s\"\n", vert_name, frag_name);
	shader = sShaderCreate();
	shader->fragment_id = fid;
	shader->vertex_id = vid;
	sShaderMake(shader);
	sShaderCacheUniforms(shader);
	return shader;
}

void sShaderLoadHL(
	char* file_name, 
	sShaderID* base,
	sShaderID* skeleton,
	sShaderID* base_shadow,
	sShaderID* skeleton_shadow
)
{
	if (base) {
		*base = sShaderLoadAndMake(
		"vertex_base", "frag_base",
		"data/shaders/vertex.glsl",
		"data/shaders/fragment.glsl",
		"#define Z_DISTANCE",
		"#define Z_DISTANCE\n#define DEFERRED");
	}
	if (skeleton) {
		*skeleton = sShaderLoadAndMake(
		"vertex_skeleton", "frag_base",
		"data/shaders/vertex.glsl",
		"data/shaders/fragment.glsl",
		"#define Z_DISTANCE\n#define SKELETON",
		"#define Z_DISTANCE\n#define DEFERRED");
	}
	if (base_shadow) {
		*base_shadow = sShaderLoadAndMake(
		"vertex_base", "frag_base_shadow",
		"data/shaders/vertex.glsl",
		"data/shaders/fragment.glsl",
		"#define Z_DISTANCE\n#define SHADOW",
		"#define Z_DISTANCE\n#define SHADOW");
	}
	if (skeleton_shadow) {
		*skeleton_shadow = sShaderLoadAndMake(
		"vertex_skeleton", "frag_base_shadow",
		"data/shaders/vertex.glsl",
		"data/shaders/fragment.glsl",
		"#define Z_DISTANCE\n#define SHADOW\n#define SKELETON",
		"#define Z_DISTANCE\n#define SHADOW");
	}
}

sShaderID sShaderMakeFromFiles(const char* name_vert, const char* name_frag)
{
	GLuint vid = sShaderLoadFromFile(name_vert, name_vert, 0, 0);
	GLuint fid = sShaderLoadFromFile(name_frag, name_frag, 1, 0);
	sShaderID shader = sShaderFindByIDs(vid, fid);
	if (shader) return shader;
	shader = sShaderCreate();
	shader->fragment_id = fid;
	shader->vertex_id = vid;
	sShaderMake(shader);
	sShaderCacheUniforms(shader);
	return shader;
}

void sShaderDelete(sShaderID shader)
{
	size_t shaders_count = sListGetSize(shaders);
	bool can_delete_program = 1;
	bool can_delete_vertex = 1;
	bool can_delete_fragment = 1;
	for (size_t i=0; i<shaders_count; i++) {
		if (shader==shaders[i]) continue;
		if (shader->program_id == shaders[i]->program_id) {
			can_delete_program = 0;
		}
		if (shader->vertex_id == shaders[i]->vertex_id) {
			can_delete_vertex = 0;
			can_delete_program = 0;
		}
		if (shader->fragment_id == shaders[i]->fragment_id) {
			can_delete_fragment = 0;
			can_delete_program = 0;
		}
	}
	
	if (!glIsProgram(shader->program_id)) return;
	if (glIsShader(shader->vertex_id) && can_delete_vertex)
	{
		glc(glDetachShader(shader->program_id, shader->vertex_id));
		glc(glDeleteShader(shader->vertex_id));
		sDictRemoveItemByValue(&shaders_ids, (void*)(uintptr_t)(shader->vertex_id+1));
	}
	if (glIsShader(shader->fragment_id) && can_delete_fragment)
	{
		glc(glDetachShader(shader->program_id,shader->fragment_id));
		glc(glDeleteShader(shader->fragment_id));
		sDictRemoveItemByValue(&shaders_ids, (void*)(uintptr_t)(shader->fragment_id+1));
	}
	if (can_delete_program) {
		glc(glDeleteProgram(shader->program_id));
	}
	for (size_t i=0; i<sListGetSize(shader->material_users); i++) {
		shader->material_users[i]->shader = 0;
	}
	for (size_t i=0; i<sListGetSize(shader->render_users); i++) {
		sListPopItem(shader->render_users[i]->shaders, shader);
	}
	sListPopItem(shaders, shader);
	sDelete(shader->material_users);
	sDelete(shader->render_users);
	sDictDelete(&shader->uniform_cache);
	sDelete(shader);
}

void sShaderDeleteDict(void)
{
	sDictDelete(&shaders_ids);
}

bool sShaderBind(sShaderID shader)
{
	bool res = shader != sShaderActive;
	if (shader) {
		shader->texture_count = 0;
		shader->point_light_count = 0;
		shader->spotlight_count = 0;
	}
	if (res) {
		sMaterialBind(0);
		if (shader) {
			glc(glUseProgram(shader->program_id));
		} else {
			glc(glUseProgram(0));
		}
	}
	sShaderActive = shader;
	return res;

	
    if (shader != sShaderActive)
    {
        sShaderActive=shader;
        glUseProgram(shader->program_id);
        glGetError();
    }
    return res;
}

bool sShaderBindTextureToID(sShaderID shader, GLuint id, sTextureID texture)
{
	sTexture null_targ = {.type=GL_TEXTURE_2D, .ID=0};
	if (!texture)
	{
		texture = &null_targ;
	}
	if (shader->texture_count>=32 || id==(GLuint)-1) return 0;
	switch (texture->type)
	{
		case GL_TEXTURE_1D : break;
		case GL_TEXTURE_2D : break;
		case GL_TEXTURE_3D : break;
		case GL_TEXTURE_1D_ARRAY : break;
		case GL_TEXTURE_2D_ARRAY : break;
		case GL_TEXTURE_RECTANGLE : break;
		case GL_TEXTURE_CUBE_MAP : break;
		case GL_TEXTURE_CUBE_MAP_ARRAY : break;
		case GL_TEXTURE_BUFFER : break;
		case GL_TEXTURE_2D_MULTISAMPLE : break;
		case GL_TEXTURE_2D_MULTISAMPLE_ARRAY : break;
		default:
			fprintf(stderr, "texture \"%s\" have incorrect type (%u). Support only GL_TEXTURE_1D, GL_TEXTURE_2D, GL_TEXTURE_3D, GL_TEXTURE_1D_ARRAY, GL_TEXTURE_2D_ARRAY, GL_TEXTURE_RECTANGLE, GL_TEXTURE_CUBE_MAP, GL_TEXTURE_CUBE_MAP_ARRAY, GL_TEXTURE_BUFFER, GL_TEXTURE_2D_MULTISAMPLE, GL_TEXTURE_2D_MULTISAMPLE_ARRAY\n", texture->name, texture->type);
			return 0;
	}
	//printf("Binding GL_TEXTURE%d \"%s\" to location sShader(%p)[%d]\n", (int)shader->texture_count, texture->name, shader, (int)uniform);
	glActiveTexture(GL_TEXTURE0 + shader->texture_count);
	glBindTexture(texture->type, texture->ID);
	glUniform1i(id, shader->texture_count);
	if (texture) {
		shader->texture_count++;
	}
	return 1;
}
bool sShaderBindTexture(sShaderID shader, char* name, sTextureID texture)
{
	if (shader->texture_count>=32) return 0;
	intptr_t uniform = sShaderGetUniformLocation(shader, name);
	if (uniform > -1) {
		return sShaderBindTextureToID(shader, uniform, texture);
	} else {
		return 0;
	}
}

void sShaderBindUniformFloatArrayToID(sShaderID shader, GLuint id, float* data, uint32_t count)
{
	switch (count)
    {
        case 2 : glUniform2fv(id,1,(const float*)data);break;
        case 3 : glUniform3fv(id,1,(const float*)data);break;
        case 4 : glUniform4fv(id,1,(const float*)data);break;
        case 9 : glUniformMatrix3fv(id,1,GL_FALSE,(const float*)data);break;
        case 12 : glUniformMatrix3x4fv(id,1,GL_FALSE,(const float*)data);break;
        case 16 : glUniformMatrix4fv(id,1,GL_FALSE,(const float*)data);break;
        default : glUniform1fv(id, count, (const float*)data);break;
    }
}
void sShaderBindUniformFloatArray(sShaderID shader, char* name, float* data, uint32_t count)
{
	if (count>16 || count<=0) return;
    intptr_t uniform = sShaderGetUniformLocation(shader, name);
	if (uniform == -1) return;
    sShaderBindUniformFloatArrayToID(shader, uniform, data, count);
}

void sShaderBindUniformFloatToID(sShaderID shader, GLuint id, float data)
{
    glUniform1f(id,data);
}
void sShaderBindUniformFloat(sShaderID shader, char* name, float data)
{
    intptr_t uniform = sShaderGetUniformLocation(shader, name);
    if (uniform==-1) return;
	glUniform1f(uniform, data);
}

void sShaderBindUniformFloat2ToID(sShaderID shader, GLuint id, float x, float y)
{
    glUniform2f(id, x, y);
}
void sShaderBindUniformFloat2(sShaderID shader, char* name, float x, float y)
{
    intptr_t uniform = sShaderGetUniformLocation(shader, name);
    if (uniform==-1) return;
    glUniform2f(uniform, x, y);
}

void sShaderBindUniformFloat3ToID(sShaderID shader, GLuint id, float x, float y, float z)
{
    glUniform3f(id, x, y, z);
}
void sShaderBindUniformFloat3(sShaderID shader, char* name, float x, float y, float z)
{
    intptr_t uniform = sShaderGetUniformLocation(shader, name);
    if (uniform==-1) return;
    glUniform3f(uniform, x, y, z);
}

void sShaderBindUniformFloat4ToID(sShaderID shader, GLuint id, float x, float y, float z, float w)
{
    glUniform4f(id, x, y, z, w);
}
void sShaderBindUniformFloat4(sShaderID shader, char* name, float x, float y, float z, float w)
{
    intptr_t uniform = sShaderGetUniformLocation(shader, name);
    if (uniform==-1) return;
    glUniform4f(uniform, x, y, z, w);
}

void sShaderBindUniformIntToID(sShaderID shader, GLuint id, int val)
{
    glUniform1i(id, val);
}
void sShaderBindUniformInt(sShaderID shader, char* name, int val)
{
    intptr_t uniform = sShaderGetUniformLocation(shader, name);
    if (uniform==-1) return;
    glUniform1i(uniform, val);
}

void sShaderBindUniformInt2ToID(sShaderID shader, GLuint id, int x, int y)
{
    glUniform2i(id, x, y);
}
void sShaderBindUniformInt2(sShaderID shader, char* name, int x, int y)
{
    intptr_t uniform = sShaderGetUniformLocation(shader, name);
    if (uniform==-1) return;
    glUniform2i(uniform, x, y);
}

void sShaderBindUniformInt3ToID(sShaderID shader, GLuint id, int x, int y, int z)
{
    glUniform3i(id, x, y, z);
}
void sShaderBindUniformInt3(sShaderID shader, char* name, int x, int y, int z)
{
    intptr_t uniform = sShaderGetUniformLocation(shader, name);
    if (uniform==-1) return;
    glUniform3i(uniform, x, y, z);
}

void sShaderBindUniformInt4ToID(sShaderID shader, GLuint id, int x, int y, int z, int w)
{
    glUniform4i(id, x, y, z, w);
}
void sShaderBindUniformInt4(sShaderID shader, char* name, int x, int y, int z, int w)
{
    intptr_t uniform = sShaderGetUniformLocation(shader, name);
    if (uniform==-1) return;
    glUniform4i(uniform, x, y, z, w);
}

void sShaderUnbindLights(sShaderID shader)
{
	for (int i=0; i<MAX_LIGHTS; i++) {
		glc(sShaderBindTextureToID(shader, shader->spotlight_vars[i][lSpotShadowmap], 0));
		glc(sShaderBindTextureToID(shader, shader->point_light_vars[i][lPointShadowmap], &(sTexture){.ID=0, .type=GL_TEXTURE_CUBE_MAP}));
	}
	glc(sShaderBindTextureToID(shader, shader->sunlight_vars[lSunShadowMap], 0));
	shader->point_light_count = 0;
	shader->spotlight_count = 0;
}

void sShaderBindLight(sShaderID shader, sGameObjectID light_object)
{
	if (!light_object || !light_object->light_component) return;
	sLightComponentID light = light_object->light_component;
	float lipos[] = {light_object->transform.global.a[3], light_object->transform.global.a[7], light_object->transform.global.a[11]};
	switch (light->type) {
		case sLightSpot : {
			laMatrix light_inv = laMul(light->projection, laInverted(light_object->transform.global));
			glc(sShaderBindUniformFloatArrayToID(shader, shader->spotlight_vars[shader->spotlight_count][lSpotColor], (float*)&light->color, 4));
			glc(sShaderBindUniformFloatArrayToID(shader, shader->spotlight_vars[shader->spotlight_count][lSpotTransform], light_inv.a, 16));
			glc(sShaderBindUniformFloatToID(shader, shader->spotlight_vars[shader->spotlight_count][lSpotBlending], light->spot_smooth));
			glc(sShaderBindUniformFloatToID(shader, shader->spotlight_vars[shader->spotlight_count][lSpotAngle], light->projection.a[0]));
			glc(sShaderBindTextureToID(shader, shader->spotlight_vars[shader->spotlight_count][lSpotShadowmap], light->shadow_buffer.color_render_targets[0]));
			glc(sShaderBindUniformFloatArrayToID(shader, shader->spotlight_vars[shader->spotlight_count][lSpotPosition], lipos, 3));
			shader->spotlight_count++;
			break;
		}
		case sLightSun : {
			laMatrix light_inv = laMul(light->projection, laInverted(light_object->transform.global));
			
			glc(sShaderBindUniformFloatArrayToID(shader, shader->sunlight_vars[lSunTransform], (float*)light_inv.a, 16));
			glc(sShaderBindUniformFloatArrayToID(shader, shader->sunlight_vars[lSunColor], (float*)&light->color, 4));
			glc(sShaderBindUniformFloatToID(shader, shader->sunlight_vars[lSunDepth], light->zfar-light->znear));
			glc(sShaderBindTextureToID(shader, shader->sunlight_vars[lSunShadowMap], light->shadow_buffer.color_render_targets[0]));
			break;
		}
		case sLightPoint : {
			glc(sShaderBindUniformFloatArrayToID(shader, shader->point_light_vars[shader->point_light_count][lPointPosition], laMatrixGetPosition(light_object->transform.global).a, 3));
			glc(sShaderBindUniformFloatArrayToID(shader, shader->point_light_vars[shader->point_light_count][lPointColor], (float*)&light->color, 4));
			glc(sShaderBindTextureToID(shader, shader->point_light_vars[shader->point_light_count][lPointShadowmap], light->shadow_buffer.color_render_targets[0]));
			shader->point_light_count++;
			break;
		}
	}
}

void sShaderBindLights(sShaderID shader, sGameObjectID* lights)
{
	int lights_count = sListGetSize(lights);
	sShaderUnbindLights(shader);
    for (size_t l=0; l<lights_count; l++) {
        glc(sShaderBindLight(shader, lights[l]));
    }
    glc(sShaderBindUniformIntToID(shader, shader->base_vars[lPointCount], shader->point_light_count));
    glc(sShaderBindUniformIntToID(shader, shader->base_vars[lSpotCount], shader->spotlight_count));
}

void sShaderAddMaterialUser(sShaderID shader, sMaterialID material)
{
	if (material && sListIndexOf(shader->material_users, material)==MAX_INDEX) {
		sListPushBack(shader->material_users, material);
	}
}

void sShaderAddRenderbufferUser(sShaderID shader, sCameraComponentID renderer)
{
	if (renderer && sListIndexOf(shader->render_users, renderer)==MAX_INDEX) {
		sListPushBack(shader->render_users, renderer);
	}
}

void sShaderRemoveMaterialUser(sShaderID shader, sMaterialID material)
{
	if (!shader) return;
	sListPopItem(shader->material_users, material);
	if (shader==material->shader) {material->shader=0;}
	if (shader==material->shader_skeleton) {material->shader_skeleton=0;}
	if (shader==material->shader_shadow) {material->shader=0;}
	if (shader==material->shader_skeleton_shadow) {material->shader_skeleton_shadow=0;}
}

void sShaderRemoveRenderbufferUser(sShaderID shader, sCameraComponentID renderer)
{
	sListPopItem(shader->render_users, renderer);
	sListPopItem(renderer->shaders, shader);
}

void sShaderRemoveUsers(sShaderID shader)
{
	while (sListGetSize(shader->render_users)) {
		sShaderRemoveRenderbufferUser(shader, shader->render_users[0]);
	}
	while (sListGetSize(shader->material_users)) {
		sShaderRemoveMaterialUser(shader, shader->material_users[0]);
	}
}

void sShaderClear(void)
{
	size_t shds_count = sListGetSize(shaders);
	sShaderID* shds = sNewArray(sShaderID, shds_count);
	memcpy(shds, shaders, sSizeof(shaders));
	for (size_t i=0; i<shds_count; i++)
	{
		if (!shds[i]->fake_user && !shds[i]->material_users && !shds[i]->render_users)
		{
			printf("Удаляется sShader(%p)\n", shds[i]);
			sShaderDelete(shds[i]);
		} else {
            printf("sShader(%p) имеет пользователей:\n", shds[i]);
			if (shds[i]->fake_user) {
				puts("  фейковый");
			}
            for (size_t m=0; m<sListGetSize(shds[i]->render_users); m++) {
                printf("  sCameraComponent(%p)\n", shds[i]->render_users[m]);
            }
            for (size_t m=0; m<sListGetSize(shds[i]->material_users); m++) {
                printf("  sMaterial(%s)\n", shds[i]->material_users[m]->name);
            }
        }
    	puts("");
	}
	sDelete(shds);
}

#ifdef __cplusplus
}
#endif
