/*
 * material.h
 *
 *  Created on: 10 июль 2020 г.
 *      Author: Ivan G
 */

#include "structures/material.h"
#include "structures/list.h"
#include "io.h"

#ifdef __cplusplus
extern "C" {
#endif

static sMaterialID* materials = 0;
static sMaterialID  sMaterialActive = 0;
extern sShaderID    sShaderActive;


sMaterialID sMaterialCreate(char* name)
{
    sMaterialID mat = sNew(sMaterial);
    strcpy(mat->name, name);
    sListPushBack(materials, mat);
    return mat;
}

sMaterialID sMaterialCreateWithDefaultShader(char* name)
{
    sMaterialID mat = sMaterialCreate(name);
    sShaderID base, skeleton, base_shadow, skeleton_shadow;
    sShaderLoadHL((char*)"data/shaders/base_frag.glsl", &base, &skeleton, &base_shadow, &skeleton_shadow);
    sMaterialSetShader(mat, base, sShaderBase);
    sMaterialSetShader(mat, skeleton, sShaderSkeleton);
    sMaterialSetShader(mat, base_shadow, sShaderBaseShadow);
    sMaterialSetShader(mat, skeleton_shadow, sShaderSkeletonShadow);
    return mat;
}

void sMaterialSetShader(sMaterialID material, sShaderID shader, sShaderType type)
{
    if (!material) return;
    sShaderID* target;
    switch (type) {
        case sShaderBase : target = &material->shader; break;
        case sShaderSkeleton : target = &material->shader_skeleton; break;
        case sShaderBaseShadow : target = &material->shader_shadow; break;
        case sShaderSkeletonShadow : target = &material->shader_skeleton_shadow; break;
        default : return;
    }
    sShaderRemoveMaterialUser(*target, material);
    if (shader) {
        sShaderAddMaterialUser(shader, material);
        *target = shader;
    }
}

void sMaterialDetachTexture(sMaterialID mat, sTextureID tex)
{
    if (tex==mat->diffuse_texture) sListPopItem(tex->material_users, mat);
    if (tex==mat->specular_texture) sListPopItem(tex->material_users, mat);
    if (tex==mat->roughness_texture) sListPopItem(tex->material_users, mat);
    if (tex==mat->metallic_texture) sListPopItem(tex->material_users, mat);
    if (tex==mat->lightmap_texture) sListPopItem(tex->material_users, mat);
    if (tex==mat->height_texture) sListPopItem(tex->material_users, mat);
}

void sMaterialDelete(sMaterialID mat)
{
    sListPopItem(materials, mat);
    sMaterialSetDiffuseMap(mat, 0);
    sMaterialSetHeightMap(mat, 0);
    sMaterialSetLightMap(mat, 0);
    sMaterialSetMetallicMap(mat, 0);
    sMaterialSetRoughnessMap(mat, 0);
    sMaterialSetSpecularMap(mat, 0);
    sMaterialSetShader(mat, 0, sShaderBase);
    sMaterialSetShader(mat, 0, sShaderSkeleton);
    sMaterialSetShader(mat, 0, sShaderBaseShadow);
    sMaterialSetShader(mat, 0, sShaderSkeletonShadow);
    sMaterialRemoveUsers(mat);
    sDelete(mat->mesh_users);
    sDelete(mat);
}

void sMaterialSetDiffuseMap(sMaterialID material, sTextureID texture) {
    if (material->diffuse_texture) {
        sListPopItem(material->diffuse_texture->material_users, material);
    }
    material->diffuse_texture = texture;
    if (texture) {
        sListPushBack(texture->material_users, material);
    }
}

void sMaterialSetSpecularMap(sMaterialID material, sTextureID texture) {
    if (material->specular_texture) {
        sListPopItem(material->specular_texture->material_users, material);
    }
    material->specular_texture = texture;
    if (texture) {
        sListPushBack(texture->material_users, material);
    }
}

void sMaterialSetRoughnessMap(sMaterialID material, sTextureID texture) {
    if (material->roughness_texture) {
        sListPopItem(material->roughness_texture->material_users, material);
    }
    material->roughness_texture = texture;
    if (texture) {
        sListPushBack(texture->material_users, material);
    }
}

void sMaterialSetMetallicMap(sMaterialID material, sTextureID texture) {
    if (material->metallic_texture) {
        sListPopItem(material->metallic_texture->material_users, material);
    }
    material->metallic_texture = texture;
    if (texture) {
        sListPushBack(texture->material_users, material);
    }
}

void sMaterialSetHeightMap(sMaterialID material, sTextureID texture) {
    if (material->height_texture) {
        sListPopItem(material->height_texture->material_users, material);
        material->height_scale = 0.0;
    }
    material->height_texture = texture;
    if (texture) {
        material->height_scale = 1.0;
        sListPushBack(texture->material_users, material);
    }
}

void sMaterialSetLightMap(sMaterialID material, sTextureID texture) {
    if (material->lightmap_texture) {
        sListPopItem(material->lightmap_texture->material_users, material);
    }
    material->lightmap_texture = texture;
    if (texture) {
        sListPushBack(texture->material_users, material);
    }
}

bool sMaterialBind(sMaterialID material)
{
    if (material==sMaterialActive) return 0;
    if (material) {
        glc(sShaderBindTextureToID(sShaderActive, sShaderActive->base_vars[fDiffuseMap], material->diffuse_texture));
        glc(sShaderBindTextureToID(sShaderActive, sShaderActive->base_vars[fSpecularMap], material->specular_texture));
        glc(sShaderBindTextureToID(sShaderActive, sShaderActive->base_vars[fReliefMap], material->height_texture));
        glc(sShaderBindTextureToID(sShaderActive, sShaderActive->base_vars[fLightMap], material->lightmap_texture));
        glc(sShaderBindTextureToID(sShaderActive, sShaderActive->base_vars[fMetallicMap], material->metallic_texture));
        glc(sShaderBindTextureToID(sShaderActive, sShaderActive->base_vars[fRoughnessMap], material->roughness_texture));

        glc(sShaderBindUniformFloat4ToID(sShaderActive, sShaderActive->base_vars[fDiffuseValue],
            material->diffuse.r, material->diffuse.g, material->diffuse.b, material->diffuse_texture!=0));
        glc(sShaderBindUniformFloatArrayToID(sShaderActive, sShaderActive->base_vars[fSpecularValue], (float*)&material->specular, 1));
        glc(sShaderBindUniformFloatToID(sShaderActive, sShaderActive->base_vars[fReliefValue], material->height_scale));
        glc(sShaderBindUniformFloatToID(sShaderActive, sShaderActive->base_vars[fMetallicValue], material->metallic_texture ? -1.0 : material->metallic));
        glc(sShaderBindUniformFloatToID(sShaderActive, sShaderActive->base_vars[fRoughnessValue], material->roughness_texture ? -1.0 : material->roughness));
        glc(sShaderBindUniformFloatToID(sShaderActive, sShaderActive->base_vars[fFresnelValue], material->fresnel));
        glc(sShaderBindUniformFloatToID(sShaderActive, sShaderActive->base_vars[fTexScrollX], material->tdx));
        glc(sShaderBindUniformFloatToID(sShaderActive, sShaderActive->base_vars[fTexScrollY], material->tdy));
    }
    sMaterialActive = material;
    return 1;
}

void sMaterialAddUser(sMaterialID material, sMeshID mesh)
{
	if (sListIndexOf(material->mesh_users, mesh)) {
		sListPushBack(material->mesh_users, mesh);
	}
}

void sMaterialRemoveUser(sMaterialID material, sMeshID mesh)
{
	sListPopItem(material->mesh_users, mesh);
    mesh->material = 0;
}

void sMaterialRemoveUsers(sMaterialID material)
{
	while (sListGetSize(material->mesh_users)) {
        sMaterialRemoveUser(material, material->mesh_users[0]);
    }
}

size_t sMaterialGetQuantity(void)
{
    return sListGetSize(materials);
}

void sMaterialClear(void)
{
    size_t mat_count = sListGetSize(materials);
    sMaterialID* mats = sNewArray(sMaterialID, mat_count);
	memcpy(mats, materials, sSizeof(materials));
	for (size_t i=0; i<mat_count; i++)
	{
		if (!mats[i]->fake_user && !mats[i]->mesh_users) {
            printf("Удаляется sMaterial(%s)\n", mats[i]->name);
			sMaterialDelete(mats[i]);
		} else {
            printf("sMaterial(%s) имеет пользователей:\n", mats[i]->name);
            for (size_t m=0; m<sListGetSize(mats[i]->mesh_users); m++) {
                printf("  sMesh(%s)\n", mats[i]->mesh_users[m]->name);
            }
        }
    	puts("");
	}
    sDelete(mats);
}

#ifdef __cplusplus
}
#endif
