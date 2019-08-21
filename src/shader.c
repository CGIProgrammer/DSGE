/*
 * shader->c
 *
 *  Created on: 22 дек. 2017 г.
 *      Author: ivan
 */

#include "engine.h"

static char GLSL_VERSION[16] = "120";

void sShaderSetVersion(char* version)
{
	strcpy(GLSL_VERSION, version);
}

char shader_log_buffer[100000];

static size_s _size_of_file(FILE* fp)
{
	size_s sz;
	fseek(fp, 0L, SEEK_END);
	sz = ftell(fp);
	fseek(fp, 0L, SEEK_SET);
	return sz;
}

extern _Bool _renderRayTracing;
void* _include_linker(char *name)
{
	FILE* fp = fopen(name,"rb");
	if (!fp)
	{
		fprintf(stderr,"%s not such file\n",name);
		exit(-1);
	}
	size_s fsize = _size_of_file(fp);
	char version[64];
	sprintf(version, " #version %s\n", GLSL_VERSION);
	char* rSSRT = "#define _SSGI\n";
	char* rSSR  = "#define _SSR\n";
	intptr_t directivesLength = 0;
	directivesLength += strlen(version);
	directivesLength += strlen(rSSRT) * sRenderGetSSGI();
	directivesLength += strlen(rSSRT) * sRenderGetReflections();
	char* buffer = sCalloc(fsize + directivesLength + 1, 1);
	strcpy(buffer, version);
	if (sRenderGetSSGI())
	{
		strcpy(buffer + strlen(buffer), rSSRT);
	}
	if (sRenderGetReflections())
	{
		strcpy(buffer + strlen(buffer), rSSR);
	}

	readf(buffer + strlen(buffer), fsize,1,fp);
	fsize = strlen(buffer);
	fclose(fp);
	_Bool incl=0;

	char* inclusion;
	while ((inclusion = strstr(buffer,"#include")) && (*(inclusion-1)=='\n' || inclusion==buffer))
	{
		if (inclusion)
		{
			//FILE* fp;
			incl = 1;
			size_s inc_size;
			char* inclusion_name;
			char* inclusion_name_end;
			char* inclusion_line_end;
			inclusion_name = strchr(inclusion,'\"')+1;
			inclusion_line_end = strchr(inclusion,'\n');
			if (inclusion_name<=inclusion)
			{
				*inclusion_line_end = 0;
				fprintf(stderr,"Inclusion error in file %s: %s\n",name,inclusion);
				exit(-2);
			}
			inclusion_name_end = strchr(inclusion_name,'\"');
			if (inclusion_name_end<=inclusion)
			{
				*inclusion_line_end = 0;
				fprintf(stderr,"Inclusion error in file %s: %s\n",name,inclusion);
				exit(-2);
			}
			*inclusion_name_end = 0;

			fp = fopen(inclusion_name,"rb");
			if (!fp)
			{
				fprintf(stderr,"Inclusion error in file %s: %s not such file\n",name,inclusion_name);
				exit(-1);
			}
			//printf("Inclusion file %s in %s\n",inclusion_name,name);
			inc_size = _size_of_file(fp);

			char* old_buf = buffer;

			buffer = sRecalloc(buffer,fsize+inc_size);
			inclusion += buffer-old_buf;
			inclusion_name += buffer-old_buf;
			inclusion_name_end += buffer-old_buf;
			inclusion_line_end += buffer-old_buf;
			fsize += inc_size;

			memmove(inclusion+inc_size,inclusion_line_end,strlen(inclusion_name_end+1));
			readf(inclusion,inc_size,1,fp);
			fclose(fp);
			fsize--;
		}
	}
	return buffer;
	if (incl) {printf("%s\n",buffer);exit(-1);}
}

static void _print_lines(char* text)
{
	unsigned line=1;
        printf("%04d ",line++);
	while (*(text++))
	{
		putchar(*text);
		if (*text=='\n')
		{
			printf("%04d ",line++);
		}
	}
}

void sShaderCompileFragment(sShader* shader)
{
	for (shader->frag_source_len=0;shader->fragment_source[shader->frag_source_len];shader->frag_source_len++);
	shader->fragment = glCreateShader(GL_FRAGMENT_SHADER);
	glShaderSource(shader->fragment, 1, (const GLchar**)&shader->fragment_source,0);
	glCompileShader(shader->fragment);

	glGetShaderiv(shader->fragment, GL_COMPILE_STATUS, &shader->success);
	if (!shader->success)
	{
		glGetShaderInfoLog(shader->fragment, sizeof(shader_log_buffer), &shader->log_len, shader_log_buffer);
		int EOLs = 1;
		printf("Shader in memory has error:\n");
		printf("%3d ",EOLs);
		for (size_s i=0;shader->fragment_source[i];i++)
		{
			printf("%c",shader->fragment_source[i]);
			if (shader->fragment_source[i]=='\n')
			{
				EOLs ++;
				printf("%3d ",EOLs);
			}
		}
		printf("\nError:\n%s\n",shader_log_buffer);
		exit(-1);
		return;
	}
}

void sShaderCompileVertex(sShader* shader)
{
	for (shader->vert_source_len=0;shader->vertex_source[shader->vert_source_len];shader->vert_source_len++);
	shader->vertex = glCreateShader(GL_VERTEX_SHADER);

	glShaderSource(shader->vertex, 1, (const GLchar**)&shader->vertex_source,0);
	glCompileShader(shader->vertex);

	glGetShaderiv(shader->vertex, GL_COMPILE_STATUS, &shader->success);
	if (!shader->success)
	{
		glGetShaderInfoLog(shader->vertex, sizeof(shader_log_buffer), &shader->log_len, shader_log_buffer);
		int EOLs = 1;
		printf("Shader in memory has error:\n");
		printf("%3d ",EOLs);
		for (size_s i=0;shader->vertex_source[i];i++)
		{
			printf("%c",shader->vertex_source[i]);
			if (shader->vertex_source[i]=='\n')
			{
				EOLs ++;
				printf("%3d ",EOLs);
			}
		}
		printf("\nError:\n%s\n",shader_log_buffer);
		exit(-1);
		return;
	}
}

void sLoadComputeFromFile(sShader* shader,const char* name)
{
	//printf("Loading shader %s\n",name);
	shader->fragment_source = _include_linker((char*)name);
	shader->compute = glCreateShader(GL_COMPUTE_SHADER);
	glShaderSource(shader->compute, 1, (const GLchar**)&shader->fragment_source,0);
	glCompileShader(shader->compute);
	glGetShaderiv(shader->compute, GL_COMPILE_STATUS, &shader->success);
	glGetShaderInfoLog(shader->compute, sizeof(shader_log_buffer), &shader->log_len, shader_log_buffer);
	if (!shader->success)
	{
		_print_lines(shader->fragment_source);
		printf("Compute shader %s has error:\n%s\n",name,shader_log_buffer);
		exit(-1);
		return;
	}
	else
	{
		//puts(shader_log_buffer);
	}

	sFree(shader->fragment_source);
	shader->program = glCreateProgram();
	glAttachShader(shader->program, shader->compute);
}

void sLoadFragmentFromFile(sShader* shader,const char* name)
{
	printf("Loading shader %s\n",name);
	shader->fragment_source = _include_linker((char*)name);
	shader->fragment = glCreateShader(GL_FRAGMENT_SHADER);
	glShaderSource(shader->fragment, 1, (const GLchar**)&shader->fragment_source,0);
	glCompileShader(shader->fragment);
	glGetShaderiv(shader->fragment, GL_COMPILE_STATUS, &shader->success);
        if (!shader->success)
        {
          puts("Shader error");
        }
	glGetShaderInfoLog(shader->fragment, sizeof(shader_log_buffer), &shader->log_len, shader_log_buffer);
	if (!shader->success)
	{
		_print_lines(shader->fragment_source);
		printf("Fragment shader %s has error:\n%s\n",name,shader_log_buffer);
		exit(-1);
		return;
	}
	else
	{
		//puts(shader_log_buffer);
	}

	sFree(shader->fragment_source);
}

void sLoadVertexFromFile(sShader* shader,const char* name)
{
	printf("Loading shader %s\n",name);
	shader->vertex_source = _include_linker((char*)name);
	shader->vertex = glCreateShader(GL_VERTEX_SHADER);
	//shader->vertex_source[shader->vert_source_len] = '\0';
	glShaderSource(shader->vertex, 1, (const GLchar**)&shader->vertex_source,0);
	glCompileShader(shader->vertex);

	//GLint link;
	glGetShaderiv(shader->vertex, GL_COMPILE_STATUS, &shader->success);
        if (!shader->success)
        {
          puts("Shader error");
        }
	glGetShaderInfoLog(shader->vertex, sizeof(shader_log_buffer), &shader->log_len, shader_log_buffer);
	if (!shader->success)
	{
		_print_lines(shader->vertex_source);
		printf("Vertex shader %s has error:\n%s\n",name,shader_log_buffer);
		exit(-1);
		return;
	}
	else
	{
		//puts(shader_log_buffer);
	}
	sFree(shader->vertex_source);
}

void sShaderMake(sShader* shader)
{
	shader->program = glCreateProgram();

	glBindAttribLocation(shader->program, 0, "pos");
	glBindAttribLocation(shader->program, 1, "nor");
	glBindAttribLocation(shader->program, 2, "uv");

	glAttachShader(shader->program,shader->vertex);
	glAttachShader(shader->program,shader->fragment);
	glLinkProgram(shader->program);
	glGetProgramiv(shader->program, GL_LINK_STATUS, &shader->success);
        if (!shader->success)
        {
          puts("Shader make error");
        }
	glGetProgramInfoLog(shader->program, sizeof(shader_log_buffer), &shader->log_len, shader_log_buffer);
	if (!shader->success)
	{
		printf("Link error: \n%s\n",shader_log_buffer);
		exit(-1);
		return;
	}
	else
	{
		//puts(shader_log_buffer);
	}
}

void sShaderValidate()
{
	glValidateProgram(activeShader);
	int log_len;
	glGetProgramInfoLog(activeShader, sizeof(shader_log_buffer), &log_len, shader_log_buffer);
	//printf("%s",shader_log_buffer);
}

void sShaderCompileMake(sShader* shader)
{
	sShaderCompileFragment(shader);
	sShaderCompileVertex(shader);
	sShaderMake(shader);
}

void sShaderCompileMakeFiles(sShader* shader,char* name_vert,char* name_frag)
{
	sLoadVertexFromFile(shader,name_vert);
	sLoadFragmentFromFile(shader,name_frag);
	sShaderMake(shader);
}

void sShaderDestroy(sShader* shader)
{
	if (!glIsProgram(shader->program)) return;
	if (glIsShader(shader->vertex))
	{
		glc(glDetachShader(shader->program,shader->vertex));
		glc(glDeleteShader(shader->vertex));
	}
	if (glIsShader(shader->fragment))
	{
		glc(glDetachShader(shader->program,shader->fragment));
		glc(glDeleteShader(shader->fragment));
	}
	glc(glDeleteProgram(shader->program));
	puts("shader destroyed\n");
	memset(shader, 0, sizeof(sShader));
}
