/*
 * mesh.c
 *
 *  Created on: 22 дек. 2017 г.
 *      Author: ivan
 */
#include "structures/mesh.h"
#include "io.h"

#ifdef __cplusplus
extern "C" {
#endif

#define MAX(a,b) ((a)>(b) ? (a) : (b))
#define MIN(a,b) ((a)<(b) ? (a) : (b))
#define BUFFER_OFFSET(i) ((void*)(i))

static sMeshID* meshes = 0;
static sMeshID last_drawn = 0;
static sShaderID last_called_shader = 0;

static const sVertex render_plane[4] =
{{-1.0,-1.0, 0.0,	0.0, 0.0, 1.0,	 0.0, 0.0, 0,0,0, 0,0,0, 0,0, 0,0,0},
{-1.0, 1.0, 0.0,	0.0, 0.0, 1.0,	 0.0, 1.0, 0,0,0, 0,0,0, 0,0, 0,0,0},
{ 1.0, 1.0, 0.0,	0.0, 0.0, 1.0,	 1.0, 1.0, 0,0,0, 0,0,0, 0,0, 0,0,0},
{ 1.0,-1.0, 0.0,	0.0, 0.0, 1.0,	 1.0, 0.0, 0,0,0, 0,0,0, 0,0, 0,0,0}};
index_t render_plane_ind[6] = {0,3,2,2,1,0};

/*static const sVertex ui_plane[4] =
{{ 0.0, 0.0, 0.0,	0.0, 0.0, 1.0,	 0.0, 0.0},
 { 0.0, 1.0, 0.0,	0.0, 0.0, 1.0,	 0.0, 1.0},
 { 1.0, 1.0, 0.0,	0.0, 0.0, 1.0,	 1.0, 1.0},
 { 1.0, 0.0, 0.0,	0.0, 0.0, 1.0,	 1.0, 0.0}};
index_t ui_plane_ind[6] = {0,3,2,2,1,0};*/

extern sShaderID sShaderActive;

/*static sMaterial default_material = {
	"default",	// name
	1.0,	// friction
	0.0,	// transparency
	0,		// glass
	0.0,	// height_scale
	{1.0,1.0,1.0,1.0}, // diffuse
	{0.0,0.0,0.0,0.0}, // specular
	0.001,	// roughness
	0.0,	// metallic
	0.0,	// fresnel
	NULL,NULL,NULL,NULL,NULL,NULL,NULL, // textures
	0.0,	// tdx
	0.0,	// tdy
	0.0,	// glow
	0.0,	// wet
	0.0,	// displacement
	0,		// shaderID
	NULL,
	1,
	0
};*/

sMeshID sMeshCreate(char* name)
{
	sMeshID mesh = sNew(sMesh);
	strcpy(mesh->name, name);
	sListPushBack(meshes, mesh);
	return mesh;
	/*
	memset(mesh, 0, sizeof(sMesh));
	strcpy(mesh->name, name);
	mesh->material = &default_material;
	sMeshMakeDynamicBuffers(mesh);*/
}

sMeshID sMeshCreateScreenPlane(void)
{
	sMeshID plane = sMeshCreate((char*)"scp");
	plane->indices = (index_t*)render_plane_ind;
	plane->ind_count = sizeof(render_plane_ind) / sizeof(render_plane_ind[0]);
	plane->vertices = (sVertex*)render_plane;
	plane->vert_count = sizeof(render_plane) / sizeof(render_plane[0]);
	sMeshMakeDynamicBuffers(plane);
	sMeshUpdateDynamicBuffers(plane, 1, 1);
	/*plane->indices = 0;
	plane->ind_count = 0;
	plane->vertices = 0;
	plane->vert_count = 0;*/
	return plane;
}

void sMeshSetMaterial(sMeshID mesh, sMaterialID material)
{
	if (mesh->material) {
		sListPopItem(mesh->material->mesh_users, mesh);
	}
	if (material) {
		sListPushBack(material->mesh_users, mesh);
	}
	mesh->material = material;
}

void sMeshMakeBuffers(sMeshID mesh)
{
	glc(glGenBuffers(1,&mesh->VBO));
	glc(glGenBuffers(1,&mesh->IBO));

	glc(glBindBuffer(GL_ARRAY_BUFFER,mesh->VBO));
	glc(glBindBuffer(GL_ELEMENT_ARRAY_BUFFER,mesh->IBO));

	glc(glBufferData(GL_ARRAY_BUFFER, sizeof(sVertex[mesh->vert_count]), mesh->vertices, GL_STATIC_DRAW));

	glc(glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(index_t[mesh->ind_count]), mesh->indices, GL_STATIC_DRAW));
}

void sMeshMakeDynamicBuffers(sMeshID mesh)
{
	glc(glGenBuffers(1,&mesh->VBO));
	glc(glGenBuffers(1,&mesh->IBO));
}

void sMeshUpdateDynamicBuffers(sMeshID mesh, bool vertices, bool indices)
{
	int size;
	glc(glBindBuffer(GL_ARRAY_BUFFER,mesh->VBO));
	glc(glGetBufferParameteriv(GL_ARRAY_BUFFER, GL_BUFFER_SIZE, &size));
	
	if (vertices)
	{
		glc(glBufferData(GL_ARRAY_BUFFER, sizeof(sVertex[mesh->vert_count]), mesh->vertices, GL_DYNAMIC_DRAW));
	}

	glc(glBindBuffer(GL_ELEMENT_ARRAY_BUFFER,mesh->IBO));
	glc(glGetBufferParameteriv(GL_ELEMENT_ARRAY_BUFFER, GL_BUFFER_SIZE, &size));
	
	if (indices)
	{
		glc(glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(index_t[mesh->ind_count]), mesh->indices, GL_DYNAMIC_DRAW));
	}
}

void sMeshDeleteBuffers(sMeshID mesh)
{
	glDeleteBuffers(1,&mesh->VBO);
	glDeleteBuffers(1,&mesh->IBO);
	mesh->IBO = mesh->VBO = 0;
}

void sMeshSetVertCoords(sMeshID mesh, laMatrix* coords, int vert_count)
{
	sFree(mesh->vertices);
	mesh->vertices = sNewArray(sVertex, vert_count);
	mesh->vert_count = vert_count;
	for (int i=0; i < vert_count; i++)
	{
		mesh->vertices[i].vx = coords->a[0];
		mesh->vertices[i].vy = coords->a[1];
		mesh->vertices[i].vz = coords->a[2];
		coords ++;
	}
	
	sMeshUpdateDynamicBuffers(mesh, 1, 0);
}

void sMeshGetVertCoords(sMeshID mesh, laMatrix* coords)
{
	for (uint32_t i=0; i < mesh->vert_count; i++)
	{
		coords[i] = Vector4(mesh->vertices[i].vx, mesh->vertices[i].vy, mesh->vertices[i].vz, 1.0);
	}
}

void sMeshSetVertTextureCoords(sMeshID mesh, laMatrix* uv, uint32_t vert_count)
{
	uint32_t vc = vert_count < mesh->vert_count ? vert_count : mesh->vert_count;

	for (uint32_t i=0; i < vc; i++)
	{
		mesh->vertices[i].u = uv->a[0];
		mesh->vertices[i].v = uv->a[1];
		uv++;
	}
	
	sMeshUpdateDynamicBuffers(mesh, 1, 0);
}

void sMeshGetVertTextureCoords(sMeshID mesh, laMatrix* uv)
{
	for (uint32_t i=0; i < mesh->vert_count; i++)
	{
		uv[i] = Vector2(mesh->vertices[i].u, mesh->vertices[i].v);
	}
}

void sMeshRecalculateNormals(sMeshID mesh)
{
	index_t* triangle = mesh->indices;

	for (uint32_t i=0; i<mesh->vert_count; i++)
	{
		sVertex* vert = mesh->vertices + i;
		vert->nx = vert->ny = vert->nz = 0;
		vert->tx = vert->ty = vert->tz = 0;
		vert->bx = vert->by = vert->bz = 0;
	}
	
	for (uint32_t i=0; i<mesh->ind_count/3; i++)
	{
		sVertex* vert1 = mesh->vertices + triangle[0];
		sVertex* vert2 = mesh->vertices + triangle[1];
		sVertex* vert3 = mesh->vertices + triangle[2];
		sVertex* verts[] = {vert1, vert2, vert3};
		laMatrix v1 = Vector(vert1->vx, vert1->vy, vert1->vz);
		laMatrix v2 = Vector(vert2->vx, vert2->vy, vert2->vz);
		laMatrix v3 = Vector(vert3->vx, vert3->vy, vert3->vz);

		laMatrix uv1 = Vector2(vert1->u, vert1->v);
		laMatrix uv2 = Vector2(vert2->u, vert2->v);
		laMatrix uv3 = Vector2(vert3->u, vert3->v);

		laMatrix deltaPos1 = laSub(v2, v1);
		laMatrix deltaPos2 = laSub(v3, v1);

		laMatrix deltaUV1 = laSub(uv2, uv1);
		laMatrix deltaUV2 = laSub(uv3, uv1);

		float r = 1.0f / (deltaUV1.a[0] * deltaUV2.a[1] - deltaUV1.a[1] * deltaUV2.a[0]);
		laMatrix normal    = laCrossn(deltaPos1, deltaPos2);
		laMatrix tangent   = laMulf(laSub(laMulf(deltaPos1, deltaUV2.a[1]), laMulf(deltaPos2, deltaUV1.a[1])), r);
        laMatrix bitangent = laMulf(laSub(laMulf(deltaPos2, deltaUV1.a[0]), laMulf(deltaPos1, deltaUV2.a[0])), r);

		for (int j=0; j<3; j++) {
			verts[j]->nx += normal.a[0];
			verts[j]->ny += normal.a[1];
			verts[j]->nz += normal.a[2];

			verts[j]->tx += tangent.a[0];
			verts[j]->ty += tangent.a[1];
			verts[j]->tz += tangent.a[2];

			verts[j]->bx += bitangent.a[0];
			verts[j]->by += bitangent.a[1];
			verts[j]->bz += bitangent.a[2];
		}
		
		triangle += 3;
	}

	for (uint32_t i=0; i<mesh->vert_count; i++)
	{
		sVertex* vert = mesh->vertices + i;
		laMatrix nor = Vector(vert->nx, vert->ny, vert->nz);
		laMatrix tan = Vector(vert->tx, vert->ty, vert->tz);
		laMatrix bin = Vector(vert->bx, vert->by, vert->bz);
		float 
		len = Length(nor);
		vert->nx /= len; vert->ny /= len; vert->nz /= len;
		len = Length(tan);
		vert->tx /= len; vert->ty /= len; vert->tz /= len;
		len = Length(bin);
		vert->bx /= len; vert->by /= len; vert->bz /= len;
	}
	
	sMeshUpdateDynamicBuffers(mesh, 1, 0);
}

void sMeshSetIndices(sMeshID mesh, int* indices, int ind_count)
{
	sFree(mesh->indices);
	mesh->indices = sNewArray(index_t, ind_count);
	mesh->ind_count = ind_count;
	memcpy(mesh->indices, indices, sizeof(int[ind_count]));

	sMeshUpdateDynamicBuffers(mesh, 0, 1);
}

void sMeshGetIndices(sMeshID mesh, int* indices)
{
	memcpy(indices, mesh->indices, sizeof(index_t[mesh->ind_count]));
}

void sMeshRecalculateBounds(sMeshID mesh)
{
	float max_x =-INFINITY, max_y =-INFINITY, max_z =-INFINITY;
	float min_x = INFINITY, min_y = INFINITY, min_z = INFINITY;
	void* verts = mesh->vertices;
	for (uint32_t i=0;i<mesh->vert_count;i++,verts=(void*)((intptr_t)verts+sizeof(sVertex)))
	{
		min_x = ((float*)verts)[0] < min_x ? ((float*)verts)[0] : min_x;
		min_y = ((float*)verts)[1] < min_y ? ((float*)verts)[1] : min_y;
		min_z = ((float*)verts)[2] < min_z ? ((float*)verts)[2] : min_z;

		max_x = ((float*)verts)[0] > max_x ? ((float*)verts)[0] : max_x;
		max_y = ((float*)verts)[1] > max_y ? ((float*)verts)[1] : max_y;
		max_z = ((float*)verts)[2] > max_z ? ((float*)verts)[2] : max_z;
	}
	mesh->bounding_box_size  = Vector(max_x-min_x, max_y-min_y, max_z-min_z);
	mesh->bounding_box_start = Vector(min_x, min_y, min_z);
	mesh->bounding_box_end   = Vector(max_x, max_y, max_z);
}

sMeshID sMeshLoad(char* name)
{
	sMeshID mesh = sMeshCreate(name);
	
	if (name)
	{
		strcpy(mesh->name, name);
	}
	
	FILE* fp = fopen(name,"rb");
	if (!fp)
	{
		fprintf(stderr, "Mesh file %s does not exist\n",name);
		exit(-1);
	}
	uint64_t l;
	readf(&l,1,8,fp); //skipping fbx hash

	readf(&mesh->deformed,1,1,fp);
	readf(&mesh->uv_count,1,1,fp);
	readf(&(mesh->ind_count),4,1,fp);
	mesh->indices = sNewArray(index_t, (size_t)mesh->ind_count);
	readf(mesh->indices,sizeof(index_t),(size_t)mesh->ind_count,fp);

	readf(&mesh->vert_count,4,1,fp);
	mesh->vertices = sNewArray(sVertex, (size_t)mesh->vert_count);
	for (uint32_t i=0;i<mesh->vert_count;i++)
	{
		readf(mesh->vertices + i, sizeof(smVertex), 1, fp);
		if (mesh->uv_count)
		{
			readf(&mesh->vertices[i].u2, sizeof(float), 1, fp);
			readf(&mesh->vertices[i].v2, sizeof(float), 1, fp);
		}
		if (mesh->deformed)
		{
			readf(&mesh->vertices[i].w1, sizeof(uint32_t), 1, fp);
			readf(&mesh->vertices[i].w2, sizeof(uint32_t), 1, fp);
			readf(&mesh->vertices[i].w3, sizeof(uint32_t), 1, fp);
		}
	}
	sMeshRecalculateBounds(mesh);

	if (!mesh->indices || !mesh->vertices)
	{
		//fprintf(LOGOUT, "Fatality\n");
		exit(-1);
	}
	uint32_t wi  = 0;
	memset(mesh->bones_indices, '\xff', sizeof(mesh->bones_indices));
	if (mesh->deformed)
	{
		//printf("\t%s has skeletal binding\n",mesh->name);
		uint32_t bone_count;
		readf(&bone_count,1,4,fp);
		mesh->link_matrix = sNewArray(laMatrix,bone_count);
		for (uint32_t i=0;i<bone_count;i++)
		{
			readf(mesh->link_matrix[i].a,1,sizeof(mesh->link_matrix[i].a),fp);
			mesh->link_matrix[i].type = MATRIX;
			//mesh->link_matrix[i] = Inverted(mesh->link_matrix[i]);
		}
		for (uint32_t i=bone_count;i<bone_count;i++)
		{
			mesh->link_matrix[i] = laIdentity;
		}
		// Преобразование весов костей в числа с правающей точкой
		for (uint32_t i=0;i<mesh->vert_count;i++)
		{
			uint32_t *bone_weights = &mesh->vertices[i].w1;
			float *bone_weights_fl = (float*)bone_weights;
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

	return mesh;
}

void sMeshAddUser(sMeshID mesh, sGameObjectID object)
{
	if (sListIndexOf(mesh->users, object)) {
		sListPushBack(mesh->users, object);
	}
}

void sMeshRemoveUser(sMeshID mesh, sGameObjectID object)
{
	sListPopItem(mesh->users, object);
	object->visual_component = 0;
}

void sMeshRemoveUsers(sMeshID mesh)
{
	while (sListGetSize(mesh->users)) {
		sMeshRemoveUser(mesh, mesh->users[0]);
	}
}

void sMeshDelete(sMeshID mesh)
{
	sMeshDeleteBuffers(mesh);
	if (mesh->indices != render_plane_ind && mesh->indices) {
		sDelete(mesh->indices);
	}
	if (mesh->vertices != render_plane && mesh->vertices) {
		sDelete(mesh->vertices);
	}
	if (mesh->vertices != render_plane && mesh->link_matrix) {
		sDelete(mesh->link_matrix);
	}

	sMeshSetMaterial(mesh, 0);
	sMeshRemoveUsers(mesh);
	
	sListPopItem(meshes, mesh);
	sDelete(mesh->users);
	sDelete(mesh);
}

void sMeshDraw(sMeshID mesh)
{
	//puts(mesh->name);
	//if (!(mesh==last_drawn && last_called_shader==sShaderActive))
	{
		glc(glBindBuffer(GL_ARRAY_BUFFER,mesh->VBO));
		glc(glBindBuffer(GL_ELEMENT_ARRAY_BUFFER,mesh->IBO));

		glc(glEnableVertexAttribArray(0));
		glBindAttribLocation(sShaderActive->program_id, 0,(char*)"pos");
		glc(glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)0));

		glc(glEnableVertexAttribArray(2));
		glBindAttribLocation(sShaderActive->program_id, 2,(char*)"uv");
		glc(glVertexAttribPointer(2, 2, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)24));

		glc(glEnableVertexAttribArray(1));
		glBindAttribLocation(sShaderActive->program_id, 1,(char*)"nor");
		glc(glVertexAttribPointer(1, 3, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)12));
		glc(glEnableVertexAttribArray(3));
		glBindAttribLocation(sShaderActive->program_id, 3,(char*)"bin");
		glc(glVertexAttribPointer(3, 3, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)32));
		glc(glEnableVertexAttribArray(4));
		glBindAttribLocation(sShaderActive->program_id, 4,(char*)"tang");
		glc(glVertexAttribPointer(4, 3, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)44));
		if (mesh->uv_count)
		{
			glc(glEnableVertexAttribArray(5));
			glc(glBindAttribLocation(sShaderActive->program_id, 5,(char*)"uv2"));
			glc(glVertexAttribPointer(5, 2, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)56));
		}
		else
		{
			glc(glEnableVertexAttribArray(5));
			glc(glBindAttribLocation(sShaderActive->program_id, 5,(char*)"uv2"));
			glc(glVertexAttribPointer(5, 2, GL_FLOAT, GL_FALSE, sizeof(sVertex), (GLvoid*)24));
		}
	}
	//glValidateProgram();
    glDrawElements(GL_TRIANGLES, mesh->ind_count, GL_UNSIGNED_INT, BUFFER_OFFSET(0));
	last_drawn = mesh;
	last_called_shader = sShaderActive;

	//glc(glBindBuffer(GL_ARRAY_BUFFER, 0));
	//glc(glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, 0));
}

void sMeshClear(void)
{
	size_t mesh_count = sListGetSize(meshes);
	sMeshID* meshes_d = sNewArray(sMeshID, mesh_count);
	memcpy(meshes_d, meshes, sSizeof(meshes));
	for (size_t i=0; i<mesh_count; i++)
	{
		if (!meshes_d[i]->fake_user && !meshes_d[i]->users)
		{
			printf("Удаляется sMesh(%s)\n", meshes_d[i]->name);
			sMeshDelete(meshes_d[i]);
		} else {
            printf("sMesh(%s) имеет пользователей:\n", meshes_d[i]->name);
			if (meshes_d[i]->fake_user) {
				puts("  фейковый");
			}
            for (size_t m=0; m<sListGetSize(meshes_d[i]->users); m++) {
                printf("  sGameObject(%s)\n", meshes_d[i]->users[m]->name);
            }
        }
    	puts("");
	}
	sDelete(meshes_d);
}

#ifdef __cplusplus
}
#endif
