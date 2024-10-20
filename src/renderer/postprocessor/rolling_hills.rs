use super::PostprocessingPass;
use super::StageIndex;
use crate::texture::TexturePixelFormat;

#[allow(dead_code)]
impl PostprocessingPass {
    /// Адаптированная демосцена Rolling Hills (https://www.shadertoy.com/view/Xsf3zX)
    pub fn rolling_hills(
        &mut self,
        width: u16,
        height: u16,
        sc_format: TexturePixelFormat,
    ) -> Result<StageIndex, String> {
        let mut stage_builder = Self::stage_builder(self.device().clone());
        stage_builder
			.dimenstions(width, height)
			//.input("image")
			//.input("accumulator")
            .output("swapchain_out", sc_format, 0)
			.code("
// Rolling hills. By David Hoskins, November 2013.
// License Creative Commons Attribution-NonCommercial-ShareAlike 3.0 Unported License.

// https://www.shadertoy.com/view/Xsf3zX

// v.2.00 Uses eiffie's 'Circle of Confusion' function
//		  for blurred ray marching into the grass.
// v.1.02 Camera aberrations.
// v.1.01 Added better grass, with wind movement.
#define THRESHOLD .003
#define MOD2 vec2(3.07965, 7.4235)
float PI  = 4.0*atan(1.0);
vec3 sunLight  = normalize( vec3(  0.35, 0.2,  0.3 ) );
vec3 cameraPos;
vec3 sunColour = vec3(1.0, .75, .6);
const mat2 rotate2D = mat2(1.932, 1.623, -1.623, 1.952);
float gTime = 0.0;
float iTime = 0.0;

//--------------------------------------------------------------------------
// Noise functions...
float Hash( float p )
{
	vec2 p2 = fract(vec2(p) / MOD2);
    p2 += dot(p2.yx, p2.xy+19.19);
	return fract(p2.x * p2.y);
}

//--------------------------------------------------------------------------
float Hash(vec2 p)
{
	p  = fract(p / MOD2);
    p += dot(p.xy, p.yx+19.19);
    return fract(p.x * p.y);
}


//--------------------------------------------------------------------------
float Noise( in vec2 x )
{
    vec2 p = floor(x);
    vec2 f = fract(x);
    f = f*f*(3.0-2.0*f);
    float n = p.x + p.y*57.0;
    float res = mix(mix( Hash(n+  0.0), Hash(n+  1.0),f.x),
                    mix( Hash(n+ 57.0), Hash(n+ 58.0),f.x),f.y);
    return res;
}

//  2 out, 2 in...
vec2 hash22(vec2 p)
{
	vec3 p3 = fract(vec3(p.xyx) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx+33.33);
    return fract((p3.xx+p3.yz)*p3.zy);

}

//--------------------------------------------------------------------------
vec2 Voronoi( in vec2 x )
{
	vec2 p = floor( x );
	vec2 f = fract( x );
	float res=100.0;
    vec2 id;
	for( int j=-1; j<=1; j++ )
	for( int i=-1; i<=1; i++ )
	{
		vec2 b = vec2( float(i), float(j) );
		vec2 r = b - f  + hash22( p + b );
		float d = dot(r,r);
		if( d < res )
		{
			res = d;
			id.x  = Hash(p+b);
		}			
    }
	return vec2(max(.4-sqrt(res), 0.0),id.x);
}




//--------------------------------------------------------------------------
vec2 Terrain( in vec2 p)
{
	float type = 0.0;
	vec2 pos = p*0.003;
	float w = 50.0;
	float f = .0;
	for (int i = 0; i < 3; i++)
	{
		f += Noise(pos) * w;
		w = w * 0.62;
		pos *= 2.5;
	}

	return vec2(f, type);
}

//--------------------------------------------------------------------------
vec2 Map(in vec3 p)
{
	vec2 h = Terrain(p.xz);
    return vec2(p.y - h.x, h.y);
}

//--------------------------------------------------------------------------
float FractalNoise(in vec2 xy)
{
	float w = .7;
	float f = 0.0;

	for (int i = 0; i < 3; i++)
	{
		f += Noise(xy) * w;
		w = w*0.6;
		xy = 2.0 * xy;
	}
	return f;
}

//--------------------------------------------------------------------------
// Grab all sky information for a given ray from camera
vec3 GetSky(in vec3 rd)
{
	float sunAmount = max( dot( rd, sunLight), 0.0 );
	float v = pow(1.0-max(rd.y,0.0),6.);
	vec3  sky = mix(vec3(.1, .2, .3), vec3(.32, .32, .32), v);
	sky = sky + sunColour * sunAmount * sunAmount * .25;
	sky = sky + sunColour * min(pow(sunAmount, 800.0)*1.5, .3);
	return clamp(sky, 0.0, 1.0);
}

//--------------------------------------------------------------------------
// Merge grass into the sky background for correct fog colouring...
vec3 ApplyFog( in vec3  rgb, in float dis, in vec3 dir)
{
	float fogAmount = clamp(dis*dis* 0.0000012, 0.0, 1.0);
	return mix( rgb, GetSky(dir), fogAmount );
}

//--------------------------------------------------------------------------
vec3 DE(vec3 p)
{
	float base = Terrain(p.xz).x - 1.9;
	float height = Noise(p.xz*2.0)*.75 + Noise(p.xz)*.35 + Noise(p.xz*.5)*.2;
	//p.y += height;
	float y = p.y - base-height;
	y = y*y;
	vec2 ret = Voronoi((p.xz*2.5+sin(y*2.0+p.zx*12.3)*.12+vec2(sin(iTime*1.3+1.5*p.z),sin(iTime*2.6+1.5*p.x))*y*.5));
	float f = ret.x * .65 + y * .5;
	return vec3( y - f*1.4, clamp(f * 1.1, 0.0, 1.0), ret.y);
}

//--------------------------------------------------------------------------
// eiffie's code for calculating the aperture size for a given distance...
float CircleOfConfusion(float t)
{
	return max(t * .04, (2.0 / iResolution.y) * (1.0+t));
}

//--------------------------------------------------------------------------
float Linstep(float a, float b, float t)
{
	return clamp((t-a)/(b-a),0.,1.);
}

//--------------------------------------------------------------------------
vec3 GrassBlades(in vec3 rO, in vec3 rD, in vec3 mat, in float dist)
{
	float d = 0.0;
	// Only calculate cCoC once is enough here...
	float rCoC = CircleOfConfusion(dist*.3);
	float alpha = 0.0;
	
	vec4 col = vec4(mat*0.15, 0.0);

	for (int i = 0; i < 15; i++)
	{
		if (col.w > .99) break;
		vec3 p = rO + rD * d;
		
		vec3 ret = DE(p);
		ret.x += .5 * rCoC;

		if (ret.x < rCoC)
		{
			alpha = (1.0 - col.y) * Linstep(-rCoC, rCoC, -ret.x);//calculate the mix like cloud density
			// Mix material with white tips for grass...
			vec3 gra = mix(mat, vec3(.35, .35, min(pow(ret.z, 4.0)*35.0, .35)), pow(ret.y, 9.0)*.7) * ret.y;
			col += vec4(gra * alpha, alpha);
		}
		d += max(ret.x * .7, .1);
	}
	if(col.w < .2)
		col.xyz = vec3(0.1, .15, 0.05);
	return col.xyz;
}

//--------------------------------------------------------------------------
// Calculate sun light...
void DoLighting(inout vec3 mat, in vec3 pos, in vec3 normal, in vec3 eyeDir, in float dis)
{
	float h = dot(sunLight,normal);
	mat = mat * sunColour*(max(h, 0.0)+.2);
}

//--------------------------------------------------------------------------
vec3 TerrainColour(vec3 pos, vec3 dir,  vec3 normal, float dis, float type)
{
	vec3 mat;
	if (type == 0.0)
	{
		// Random colour...
		mat = mix(vec3(.0,.3,.0), vec3(.2,.3,.0), Noise(pos.xz*.025));
		// Random shadows...
		float t = FractalNoise(pos.xz * .1)+.5;
		// Do grass blade tracing...
		mat = GrassBlades(pos, dir, mat, dis) * t;
		DoLighting(mat, pos, normal,dir, dis);
	}
	mat = ApplyFog(mat, dis, dir);
	return mat;
}

//--------------------------------------------------------------------------
// Home in on the surface by dividing by two and split...
// Note, this does a much better job than using 'abs' on the distance march.
float BinarySubdivision(in vec3 rO, in vec3 rD, float t, float oldT)
{
	float halfwayT = 0.0;
	for (int n = 0; n < 5; n++)
	{
		halfwayT = (oldT + t ) * .5;
        float h = Map(rO + halfwayT*rD).x;
        (h < THRESHOLD) ? t = halfwayT:oldT = halfwayT;
  }
	return t;
}

//--------------------------------------------------------------------------
bool Scene(in vec3 rO, in vec3 rD, out float resT, out float type )
{
    float t = 5.;
	float oldT = 0.0;
	float delta = 0.;
	float h = 0.0;
	bool hit = false;
	for( int j=0; j < 60; j++ )
	{
	    vec3 p = rO + t*rD;
		h = Map(p).x; // ...Get this position's height mapping.

		// Are we inside, and close enough to fudge a hit?...
		if( h < THRESHOLD)
		{
			hit = true;
            break;
		}
        oldT = t;
		t +=  h + (t*0.04);
	}
    type = 0.0;
    resT = BinarySubdivision(rO, rD, t, oldT);
	return hit;
}

//--------------------------------------------------------------------------
vec3 CameraPath( float t )
{
	//t = time + t;
    vec2 p = vec2(200.0 * sin(3.54*t), 200.0 * cos(2.0*t) );
	return vec3(p.x+55.0,  12.0+sin(t*.3)*6.5, -94.0+p.y);
} 

//--------------------------------------------------------------------------
vec3 PostEffects(vec3 rgb, vec2 xy)
{
	// Gamma first...
	rgb = pow(rgb, vec3(0.45));
	
	// Then...
	#define CONTRAST 1.1
	#define SATURATION 1.3
	#define BRIGHTNESS 1.3
	rgb = mix(vec3(.5), mix(vec3(dot(vec3(.2125, .7154, .0721), rgb*BRIGHTNESS)), rgb*BRIGHTNESS, SATURATION), CONTRAST);
	// Vignette...
	rgb *= .4+0.5*pow(40.0*xy.x*xy.y*(1.0-xy.x)*(1.0-xy.y), 0.2 );	
	return rgb;
}

//--------------------------------------------------------------------------
void main()
{
	float gTime = (timer.uptime*5.0+2352.0)*.006;
    vec2 xy = fragCoordWp.xy;
    xy.y = 1.0 - xy.y;
	vec2 uv = (-1.0 + 2.0 * xy) * vec2(iResolution.x/iResolution.y,1.0);
	vec3 camTar;

	cameraPos = CameraPath(gTime + 0.0);
    cameraPos.x -= 3.0;
	camTar	 = CameraPath(gTime + .009);
	cameraPos.y += Terrain(CameraPath(gTime + .009).xz).x;
	camTar.y = cameraPos.y;
	
	float roll = .4*sin(gTime+.5);
	vec3 cw = normalize(camTar-cameraPos);
	vec3 cp = vec3(sin(roll), cos(roll),0.0);
	vec3 cu = cross(cw,cp);
	vec3 cv = cross(cu,cw);
	vec3 dir = normalize(uv.x*cu + uv.y*cv + 1.3*cw);
	mat3 camMat = mat3(cu, cv, cw);

	#ifdef STEREO
	cameraPos += .85*cu*isCyan; // move camera to the right - the rd vector is still good
	#endif

	vec3 col;
	float distance;
	float type;
	if( !Scene(cameraPos, dir, distance, type) )
	{
		// Missed scene, now just get the sky...
		col = GetSky(dir);
	}
	else
	{
		// Get world coordinate of landscape...
		vec3 pos = cameraPos + distance * dir;
		// Get normal from sampling the high definition height map
		// Use the distance to sample larger gaps to help stop aliasing...
		vec2 p = vec2(0.1, 0.0);
		vec3 nor  	= vec3(0.0,		Terrain(pos.xz).x, 0.0);
		vec3 v2		= nor-vec3(p.x,	Terrain(pos.xz+p).x, 0.0);
		vec3 v3		= nor-vec3(0.0,	Terrain(pos.xz-p.yx).x, -p.x);
		nor = cross(v2, v3);
		nor = normalize(nor);

		// Get the colour using all available data...
		col = TerrainColour(pos, dir, nor, distance, type);
	}
	
	// bri is the brightness of sun at the centre of the camera direction.
	// Yeah, the lens flares is not exactly subtle, but it was good fun making it.
	float bri = dot(cw, sunLight)*.75;
	if (bri > 0.0)
	{
		vec2 sunPos = vec2( dot( sunLight, cu ), dot( sunLight, cv ) );
		vec2 uvT = uv-sunPos;
		uvT = uvT*(length(uvT));
		bri = pow(bri, 6.0)*.8;

		// glare = the red shifted blob...
		float glare1 = max(dot(normalize(vec3(dir.x, dir.y+.3, dir.z)),sunLight),0.0)*1.4;
		// glare2 is the yellow ring...
		float glare2 = max(1.0-length(uvT+sunPos*.5)*4.0, 0.0);
		uvT = mix (uvT, uv, -2.3);
		// glare3 is a purple splodge...
		float glare3 = max(1.0-length(uvT+sunPos*5.0)*1.2, 0.0);

		col += bri * vec3(1.0, .0, .0)  * pow(glare1, 12.5)*.05;
		col += bri * vec3(1.0, 1.0, 0.2) * pow(glare2, 2.0)*2.5;
		col += bri * sunColour * pow(glare3, 2.0)*3.0;
	}
	col = PostEffects(col, xy);	
	
	#ifdef STEREO	
	col *= vec3( isCyan, 1.0-isCyan, 1.0-isCyan );	
	#endif
	
	swapchain_out = vec4(pow(col, vec3(2.2)),1.0);
}");
        let shader = stage_builder.build(self);
        shader
    }
}
