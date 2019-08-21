
#ifdef VERTEX
#extension GL_ARB_explicit_attrib_location : enable
#ifdef GL_ARB_explicit_attrib_location
attribute vec3 pos;
attribute vec3 nor;
attribute vec2 uv;
attribute vec3 bin;
attribute vec3 tang;
#ifdef SKELETON
attribute vec3 weights;
#endif
attribute vec2 uv2;
#else
layout (location=0) in vec3 pos;
layout (location=1) in vec3 nor;
layout (location=2) in vec2 uv;
layout (location=3) in vec3 bin;
layout (location=4) in vec3 tang;
layout (location=5) in vec3 weights;
layout (location=6) in vec2 uv2;
#endif
#endif
