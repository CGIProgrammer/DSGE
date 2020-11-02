void pbr(mat3 tbn)
{
    mDiffuse = mix(vec4(fDiffuseValue.rgb, 1.0), texture(fDiffuseMap, texture_uv), fDiffuseValue.a);
    vec3 nrm = texture(fReliefMap, texture_uv).rgb;
    mNormal = mix(vec3(0.0,0.0,1.0), (nrm*2.0-1.0)*vec3(1.0, -1.0, 1.0), clamp(fReliefValue, 0.0, 1.0));
    mNormal *= tbn;
    mSpecular = texture(fSpecularMap, texture_uv).r * fSpecularValue;
    if (fRoughnessValue<0.0) {
        mRoughness = texture(fRoughnessMap, texture_uv).r;
    } else {
        mRoughness = fRoughnessValue;
    }
    if (fMetallicValue<0.0) {
        mMetallic = texture(fMetallicMap, texture_uv).r;
    } else {
        mMetallic = fMetallicValue;
    }
    mMetallic = max(texture(fMetallicMap, texture_uv).r, fMetallicValue);
    mAmbient = vec3(0.0); //texture(fLightMap, texture_uv).rgb;
}