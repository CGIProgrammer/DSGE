/*
 * mesh.c
 *
 *  Created on: 22 дек. 2017 г.
 *      Author: ivan
 */
#include "engine.h"
#define HASH(var) _hash(&var,sizeof(var))


uint32_t S_Name2hash(char* name)
{
	uint32_t result=0;
	uint64_t a;
	for (size_t i=0;name[i];i++)
	{
		a = result;
		a += (a<<5)^(unsigned char)name[i];
		result = a ^ (a>>17);
	}
	return result;
}

void getFileList(char* dir_name,S_NAME_ARRAY* names)
{
	DIR *dir;
	struct dirent *ent;
	dir = opendir((const char*)dir_name);
	for (names->count=0;readdir(dir)!=0;names->count++);
	closedir(dir);
	names->names = sCalloc(names->count,sizeof(S_NAME));
	printf("%d names\n",names->count);
	dir = opendir((const char*)dir_name);
	for (uint32_t i=0;i<names->count;i++)
	{
		ent = readdir(dir);
		strcpy(&(names->names[i].name[0]),ent->d_name);
		names->names[i].hash = S_Name2hash(names->names[i].name);
	}
	closedir(dir);
}

void sMeshMakeBuffers(sMesh* mesh)
{
	glc(glGenBuffers(1,&mesh->VBO));
	glc(glGenBuffers(1,&mesh->IBO));

	glc(glBindBuffer(GL_ARRAY_BUFFER,mesh->VBO));
	glc(glBindBuffer(GL_ELEMENT_ARRAY_BUFFER,mesh->IBO));

	glc(glBufferData(GL_ARRAY_BUFFER, (sizeof(sVertex)+mesh->deformed*12+mesh->uv2*8)*mesh->vert_count, mesh->vertices, GL_STATIC_DRAW));

	glc(glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(index_t)*mesh->ind_count, mesh->indices, GL_STATIC_DRAW));

}

void sMeshDeleteBuffers(sMesh* mesh)
{
	glDeleteBuffers(1,&mesh->VBO);
	glDeleteBuffers(1,&mesh->IBO);
}

void sMeshDraw(sMesh* mesh)
{
	glBindBuffer(GL_ARRAY_BUFFER,mesh->VBO);
	glBindBuffer(GL_ELEMENT_ARRAY_BUFFER,mesh->IBO);

	mesh->uniforms[0] = glGetProgramResourceLocation(mesh->material->shader,GL_UNIFORM,"orientation");
	mesh->uniforms[1] = glGetProgramResourceLocation(mesh->material->shader,GL_UNIFORM,"material_diffuse");
	mesh->uniforms[2] = glGetProgramResourceLocation(mesh->material->shader,GL_UNIFORM,"material_specular");
	glProgramUniformMatrix4fv(mesh->material->shader,mesh->uniforms[0],1,GL_FALSE,(GLfloat*)&mesh->transform);
	glProgramUniform3fv(mesh->material->shader,mesh->uniforms[1],1,(GLfloat*)&mesh->material->diffuse);
	glProgramUniform3fv(mesh->material->shader,mesh->uniforms[2],1,(GLfloat*)&mesh->material->specular);

	glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 8 * sizeof(GLfloat), (GLvoid*)0);
	glEnableVertexAttribArray(0);
	glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, 8 * sizeof(GLfloat), (GLvoid*)12);
	glEnableVertexAttribArray(1);
	glVertexAttribPointer(2, 2, GL_FLOAT, GL_FALSE, 8 * sizeof(GLfloat), (GLvoid*)24);
	glEnableVertexAttribArray(2);
	glDrawElements(GL_TRIANGLES,mesh->ind_count,GL_UNSIGNED_SHORT,BUFFER_OFFSET(0));
}

void sMeshLoad(sMesh* mesh, sMaterial* mat,char* name)
{
	char filename[256];
	//memset(mesh, 0, sizeof(sMesh));
	if (name)
	{
		strcpy(mesh->name, name);
		mesh->hash = S_Name2hash(name);
	}
	if (mat)
	{
		mesh->material = mat;
	}
	sprintf(filename,RESDIR"/mesh/%s"RESMESH,name);
	FILE* fp = fopen(filename,"rb");
	if (!fp)
	{
		fprintf(LOGOUT, "Mesh file %s does not exist\n",filename);
		exit(-1);
	}
	uint64_t l;
	readf(&l,1,8,fp); //skipping fbx hash

	readf(&mesh->deformed,1,1,fp);
	readf(&mesh->uv2,1,1,fp);
	readf(&(mesh->ind_count),4,1,fp);
	mesh->indices = sCalloc(sizeof(index_t),(size_s)mesh->ind_count);
	readf(mesh->indices,sizeof(index_t),(size_s)mesh->ind_count,fp);

	readf(&mesh->vert_count,4,1,fp);
	mesh->vertices = sCalloc(sizeof(sVertex)+mesh->deformed*12+mesh->uv2*8,(size_s)mesh->vert_count);
	readf(mesh->vertices,sizeof(sVertex)+mesh->deformed*12+mesh->uv2*8,(size_s)mesh->vert_count,fp);

	void* verts = mesh->vertices;
	float max_x,max_y,max_z;
	float min_x,min_y,min_z;
	max_x = max_y = max_z =-INFINITY;
	min_x = min_y = min_z = INFINITY;
	for (uint32_t i=0;i<mesh->vert_count;i++,verts+=sizeof(sVertex)+mesh->deformed*12+mesh->uv2*8)
	{
		min_x = ((float*)verts)[0] < min_x ? ((float*)verts)[0] : min_x;
		min_y = ((float*)verts)[1] < min_y ? ((float*)verts)[1] : min_y;
		min_z = ((float*)verts)[2] < min_z ? ((float*)verts)[2] : min_z;

		max_x = ((float*)verts)[0] > max_x ? ((float*)verts)[0] : max_x;
		max_y = ((float*)verts)[1] > max_y ? ((float*)verts)[1] : max_y;
		max_z = ((float*)verts)[2] > max_z ? ((float*)verts)[2] : max_z;
	}

	mesh->bounding_box.a[0] = max_x-min_x;
	mesh->bounding_box.a[1] = max_y-min_y;
	mesh->bounding_box.a[2] = max_z-min_z;
	mesh->bounding_box.type = VECTOR;

	if (!mesh->indices || !mesh->vertices)
	{
		fprintf(LOGOUT, "Fatality\n");
		exit(-1);
	}
	uint32_t wi  = 0;
	memset(mesh->bones_indices, '\xff', sizeof(mesh->bones_indices));
	if (mesh->deformed)
	{
		//printf("\t%s has skeletal binding\n",mesh->name);
		uint32_t bone_count;
		readf(&bone_count,1,4,fp);
		mesh->link_matrix = sCalloc(sizeof(laType),bone_count);
		for (uint32_t i=0;i<bone_count;i++)
		{
			readf(mesh->link_matrix[i].a,1,sizeof(mesh->link_matrix[i].a),fp);
			mesh->link_matrix[i].type = MATRIX;
			//mesh->link_matrix[i] = Inverted(mesh->link_matrix[i]);
		}
		for (uint32_t i=bone_count;i<bone_count;i++)
		{
			mesh->link_matrix[i] = Identity;
		}
		// Преобразование весов костей в числа с правающей точкой
		uint32_t vertex_size = sizeof(sVertex)+sizeof(float[3])+mesh->uv2*sizeof(float[2]);
		void *verts = mesh->vertices;
		for (uint32_t i=0;i<mesh->vert_count;i++,verts+=vertex_size)
		{
			uint32_t *bone_weights = (void*)verts + sizeof(sVertex);
			float *bone_weights_fl = (void*)bone_weights;
			uint32_t wi1 = bone_weights[0]>>16;
			uint32_t wi2 = bone_weights[1]>>16;
			uint32_t wi3 = bone_weights[2]>>16;
			uint32_t ww1 = bone_weights[0] & 0xFFFF;
			uint32_t ww2 = bone_weights[1] & 0xFFFF;
			uint32_t ww3 = bone_weights[2] & 0xFFFF;
			for (wi=0; mesh->bones_indices[wi]!=0xFFFF && mesh->bones_indices[wi] != wi1; wi++);
			if (wi>=128)
			{
				fprintf(stderr, "Too many bones per one deformable mesh %s\n",mesh->name);
				exit(-1);
			}
			if (mesh->bones_indices[wi] == 0xFFFF && wi<128)
			{
				mesh->bones_indices[wi] = wi1;
			}
			bone_weights_fl[0] = wi + MIN(ww1/65536.0, 0.9999);

			for (wi=0; mesh->bones_indices[wi]!=0xFFFF && mesh->bones_indices[wi] != wi2; wi++);
			if (mesh->bones_indices[wi] == 0xFFFF && wi<128)
			{
				mesh->bones_indices[wi] = wi2;
			}
			bone_weights_fl[1] = wi + MIN(ww2/65536.0, 0.9999);

			for (wi=0; mesh->bones_indices[wi]!=0xFFFF && mesh->bones_indices[wi] != wi3; wi++);
			if (mesh->bones_indices[wi] == 0xFFFF && wi<128)
			{
				mesh->bones_indices[wi] = wi3;
			}
			bone_weights_fl[2] = wi + MIN(ww3/65536.0, 0.9999);
		}
	}

	fclose(fp);
	glc(sMeshMakeBuffers(mesh));
	sFree(mesh->indices);
	sFree(mesh->vertices);
	mesh->indices = 0;
	mesh->vertices = 0;
}

void sMeshDelete(sMesh* mesh)
{
	sMeshDeleteBuffers(mesh);
}


