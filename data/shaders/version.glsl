 //#version 330
#extension GL_ARB_gpu_shader5 : enable
#if __VERSION__ != 100 && __VERSION__ != 110 && __VERSION__ != 120
  #define input in
  #define output out
#else
  #define input varying
  #define output varying
#endif
