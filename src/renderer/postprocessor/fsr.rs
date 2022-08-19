use super::PostprocessingPass;
use crate::texture::{TexturePixelFormat, TextureFilter, TextureView};
use super::{StageIndex, StageOutputIndex, StageInputIndex};

#[allow(dead_code)]
impl PostprocessingPass
{
    /// Адаптированный Super Resolution v1.0 от AMD
    pub fn fidelityfx_super_resolution(&mut self, width: u16, height: u16) -> ((StageIndex, StageInputIndex), (StageIndex, StageOutputIndex))
    {
        let easu = self.make_easu_stage(width, height).unwrap();
        let rcas = self.make_rcas_stage(width, height).unwrap();
        self.link_stages(easu, 0, rcas, "EASU_pass".to_owned());
        ((easu, "albedo".to_owned()), (rcas, 0))
    }
    
    fn make_easu_stage(&mut self, width: u16, height: u16) -> Result<StageIndex, String>
    {
        let mut stage_builder = Self::stage_builder(self._device.clone());
        stage_builder
            .dimenstions(width, height)
            .input("albedo", TextureView::Dim2d)
            .output("EASU_pass", TexturePixelFormat::R8G8B8A8_UNORM, TextureFilter::Nearest, false)
            .code("
            // Copyright (c) 2021 Advanced Micro Devices, Inc. All rights reserved.
            // Permission is hereby granted, free of charge, to any person obtaining a copy
            // of this software and associated documentation files(the \"Software\"), to deal
            // in the Software without restriction, including without limitation the rights
            // to use, copy, modify, merge, publish, distribute, sublicense, and / or sell
            // copies of the Software, and to permit persons to whom the Software is
            // furnished to do so, subject to the following conditions :
            // The above copyright notice and this permission notice shall be included in
            // all copies or substantial portions of the Software.
            // THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
            // IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
            // FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.IN NO EVENT SHALL THE
            // AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
            // LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
            // OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
            // THE SOFTWARE.

            vec3 FsrEasuCF(vec2 p) {
                return texelFetch(albedo, ivec2(p*vec2(textureSize(albedo, 0))), 0).rgb;
            }
            
            void FsrEasuCon(
                out vec4 con0,
                out vec4 con1,
                out vec4 con2,
                out vec4 con3,
                vec2 inputViewportInPixels,
                vec2 inputSizeInPixels,
                vec2 outputSizeInPixels
            )
            {
                con0 = vec4(
                    inputViewportInPixels.x/outputSizeInPixels.x,
                    inputViewportInPixels.y/outputSizeInPixels.y,
                    .5*inputViewportInPixels.x/outputSizeInPixels.x-.5,
                    .5*inputViewportInPixels.y/outputSizeInPixels.y-.5
                );
                
                con1 = vec4(1,1,1,-1)/inputSizeInPixels.xyxy;
                
                con2 = vec4(-1,2,1,2)/inputSizeInPixels.xyxy;
                con3 = vec4(0,4,0,0)/inputSizeInPixels.xyxy;
            }
            
            // Filtering for a given tap for the scalar.
            void FsrEasuTapF(
                inout vec3 aC, // Accumulated color, with negative lobe.
                inout float aW, // Accumulated weight.
                vec2 off, // Pixel offset from resolve position to tap.
                vec2 dir, // Gradient direction.
                vec2 len, // Length.
                float lob, // Negative lobe strength.
                float clp, // Clipping point.
                vec3 c
            )
            {
                
                vec2 v = vec2(dot(off, dir), dot(off,vec2(-dir.y,dir.x)));
                
                v *= len;
                
                float d2 = min(dot(v,v),clp);
                
                float wB = .4 * d2 - 1.;
                float wA = lob * d2 -1.;
                wB *= wB;
                wA *= wA;
                wB = 1.5625*wB-.5625;
                float w=  wB * wA;
                
                aC += c*w;
                aW += w;
            }

            void FsrEasuSetF(
                inout vec2 dir,
                inout float len,
                float w,
                float lA,float lB,float lC,float lD,float lE
            )
            {
                float lenX = max(abs(lD - lC), abs(lC - lB));
                float dirX = lD - lB;
                dir.x += dirX * w;
                lenX = clamp(abs(dirX)/lenX,0.,1.);
                lenX *= lenX;
                len += lenX * w;
                
                float lenY = max(abs(lE - lC), abs(lC - lA));
                float dirY = lE - lA;
                dir.y += dirY * w;
                lenY = clamp(abs(dirY) / lenY,0.,1.);
                lenY *= lenY;
                len += lenY * w;
            }
            
            void FsrEasuF(
                out vec3 pix,
                vec2 ip, // Integer pixel position in output.
                // Constants generated by FsrEasuCon().
                vec4 con0, // xy = output to input scale, zw = first pixel offset correction
                vec4 con1,
                vec4 con2,
                vec4 con3
            )
            {
                vec2 pp = ip * con0.xy + con0.zw; // Corresponding input pixel/subpixel
                vec2 fp = floor(pp);// fp = source nearest pixel
                pp -= fp; // pp = source subpixel
            
                vec2 p0 = fp * con1.xy + con1.zw;
                
                vec2 p1 = p0 + con2.xy;
                vec2 p2 = p0 + con2.zw;
                vec2 p3 = p0 + con3.xy;
            
                vec4 off = vec4(-.5,.5,-.5,.5)*con1.xxyy;
                
                vec3 bC = FsrEasuCF(p0 + off.xw); float bL = bC.g + 0.5 *(bC.r + bC.b);
                vec3 cC = FsrEasuCF(p0 + off.yw); float cL = cC.g + 0.5 *(cC.r + cC.b);
                vec3 iC = FsrEasuCF(p1 + off.xw); float iL = iC.g + 0.5 *(iC.r + iC.b);
                vec3 jC = FsrEasuCF(p1 + off.yw); float jL = jC.g + 0.5 *(jC.r + jC.b);
                vec3 fC = FsrEasuCF(p1 + off.yz); float fL = fC.g + 0.5 *(fC.r + fC.b);
                vec3 eC = FsrEasuCF(p1 + off.xz); float eL = eC.g + 0.5 *(eC.r + eC.b);
                vec3 kC = FsrEasuCF(p2 + off.xw); float kL = kC.g + 0.5 *(kC.r + kC.b);
                vec3 lC = FsrEasuCF(p2 + off.yw); float lL = lC.g + 0.5 *(lC.r + lC.b);
                vec3 hC = FsrEasuCF(p2 + off.yz); float hL = hC.g + 0.5 *(hC.r + hC.b);
                vec3 gC = FsrEasuCF(p2 + off.xz); float gL = gC.g + 0.5 *(gC.r + gC.b);
                vec3 oC = FsrEasuCF(p3 + off.yz); float oL = oC.g + 0.5 *(oC.r + oC.b);
                vec3 nC = FsrEasuCF(p3 + off.xz); float nL = nC.g + 0.5 *(nC.r + nC.b);
               
                vec2 dir = vec2(0);
                float len = 0.;
            
                FsrEasuSetF(dir, len, (1.-pp.x)*(1.-pp.y), bL, eL, fL, gL, jL);
                FsrEasuSetF(dir, len,    pp.x  *(1.-pp.y), cL, fL, gL, hL, kL);
                FsrEasuSetF(dir, len, (1.-pp.x)*  pp.y  , fL, iL, jL, kL, nL);
                FsrEasuSetF(dir, len,    pp.x  *  pp.y  , gL, jL, kL, lL, oL);
            
                vec2 dir2 = dir * dir;
                float dirR = dir2.x + dir2.y;
                bool zro = dirR < (1.0/32768.0);
                dirR = inversesqrt(dirR);
                dirR = zro ? 1.0 : dirR;
                dir.x = zro ? 1.0 : dir.x;
                dir *= vec2(dirR);
                
                len = len * 0.5;
                len *= len;
                
                float stretch = dot(dir,dir) / (max(abs(dir.x), abs(dir.y)));
                vec2 len2 = vec2(1. +(stretch-1.0)*len, 1. -.5 * len);
                float lob = .5 - .29 * len;
                float clp = 1./lob;
                
                vec3 min4 = min(min(fC,gC),min(jC,kC));
                vec3 max4 = max(max(fC,gC),max(jC,kC));
                // Accumulation.
                vec3 aC = vec3(0);
                float aW = 0.;
                FsrEasuTapF(aC, aW, vec2( 0,-1)-pp, dir, len2, lob, clp, bC);
                FsrEasuTapF(aC, aW, vec2( 1,-1)-pp, dir, len2, lob, clp, cC);
                FsrEasuTapF(aC, aW, vec2(-1, 1)-pp, dir, len2, lob, clp, iC);
                FsrEasuTapF(aC, aW, vec2( 0, 1)-pp, dir, len2, lob, clp, jC);
                FsrEasuTapF(aC, aW, vec2( 0, 0)-pp, dir, len2, lob, clp, fC);
                FsrEasuTapF(aC, aW, vec2(-1, 0)-pp, dir, len2, lob, clp, eC);
                FsrEasuTapF(aC, aW, vec2( 1, 1)-pp, dir, len2, lob, clp, kC);
                FsrEasuTapF(aC, aW, vec2( 2, 1)-pp, dir, len2, lob, clp, lC);
                FsrEasuTapF(aC, aW, vec2( 2, 0)-pp, dir, len2, lob, clp, hC);
                FsrEasuTapF(aC, aW, vec2( 1, 0)-pp, dir, len2, lob, clp, gC);
                FsrEasuTapF(aC, aW, vec2( 1, 2)-pp, dir, len2, lob, clp, oC);
                FsrEasuTapF(aC, aW, vec2( 0, 2)-pp, dir, len2, lob, clp, nC);
                
                pix=min(max4,max(min4,aC/aW));
            }
            
            void main()
            {
                vec3 c;
                vec4 con0,con1,con2,con3;
                
                vec2 rendersize = vec2(textureSize(albedo, 0)); //iChannelResolution[0].xy;
                FsrEasuCon(
                    con0, con1, con2, con3, rendersize, rendersize, iResolution.xy
                );
                FsrEasuF(c, fragCoord, con0, con1, con2, con3);
                EASU_pass = vec4(c.xyz, 1.0);
            }");
            stage_builder.build(self)
    }

    fn make_rcas_stage(&mut self, width: u16, height: u16) -> Result<StageIndex, String>
    {
        let mut stage_builder = Self::stage_builder(self._device.clone());
        stage_builder
            .dimenstions(width, height)
            .input("EASU_pass", TextureView::Dim2d)
            .output("fsr_out", TexturePixelFormat::B8G8R8A8_SRGB, TextureFilter::Nearest, false)
            .code("
            // Copyright (c) 2021 Advanced Micro Devices, Inc. All rights reserved.
            // Permission is hereby granted, free of charge, to any person obtaining a copy
            // of this software and associated documentation files(the \"Software\"), to deal
            // in the Software without restriction, including without limitation the rights
            // to use, copy, modify, merge, publish, distribute, sublicense, and / or sell
            // copies of the Software, and to permit persons to whom the Software is
            // furnished to do so, subject to the following conditions :
            // The above copyright notice and this permission notice shall be included in
            // all copies or substantial portions of the Software.
            // THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
            // IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
            // FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.IN NO EVENT SHALL THE
            // AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
            // LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
            // OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
            // THE SOFTWARE.
            
            #define FSR_RCAS_LIMIT (0.25-(1.0/16.0))
            #define FSR_RCAS_DENOISE

            vec4 FsrRcasLoadF(vec2 p);
            void FsrRcasCon(
                out float con,
                float sharpness
            ){
                con = exp2(-sharpness);
            }

            vec3 FsrRcasF(
                vec2 ip,
                float con
            )
            {
                vec2 sp = vec2(ip);
                vec3 b = FsrRcasLoadF(sp + vec2( 0,-1)).rgb;
                vec3 d = FsrRcasLoadF(sp + vec2(-1, 0)).rgb;
                vec3 e = FsrRcasLoadF(sp).rgb;
                vec3 f = FsrRcasLoadF(sp+vec2( 1, 0)).rgb;
                vec3 h = FsrRcasLoadF(sp+vec2( 0, 1)).rgb;
                
                float bL = b.g + .5 * (b.b + b.r);
                float dL = d.g + .5 * (d.b + d.r);
                float eL = e.g + .5 * (e.b + e.r);
                float fL = f.g + .5 * (f.b + f.r);
                float hL = h.g + .5 * (h.b + h.r);
                
                float nz = .25 * (bL + dL + fL + hL) - eL;
                nz=clamp(
                    abs(nz)
                    /(
                        max(max(bL,dL),max(eL,max(fL,hL)))
                        -min(min(bL,dL),min(eL,min(fL,hL)))
                    ),
                    0., 1.
                );
                nz=1.-.5*nz;
                
                vec3 mn4 = min(b, min(f, h));
                vec3 mx4 = max(b, max(f, h));
                
                vec2 peakC = vec2(1., -4.);
                
                vec3 hitMin = mn4 / (4. * mx4);
                vec3 hitMax = (peakC.x - mx4) / (4.* mn4 + peakC.y);
                vec3 lobeRGB = max(-hitMin, hitMax);
                float lobe = max(
                    -FSR_RCAS_LIMIT,
                    min(max(lobeRGB.r, max(lobeRGB.g, lobeRGB.b)), 0.)
                )*con;
                
                #ifdef FSR_RCAS_DENOISE
                lobe *= nz;
                #endif
                
                return (lobe * (b + d + h + f) + e) / (4. * lobe + 1.);
            } 


            vec4 FsrRcasLoadF(vec2 p) {
                return texelFetch(EASU_pass, ivec2(p), 0);
            }

            void main()
            {
                vec2 uv = fragCoord/iResolution.xy;
                
                float con;
                float sharpness = 0.0;

                FsrRcasCon(con,sharpness);

                vec3 col = FsrRcasF(fragCoord-vec2(1.0), con);

                fsr_out = vec4(col, 1.0);
            }");
            stage_builder.build(self)
    }
}