//digits
int number[]=int[]( 0x1d18c62e,
                    0x2328421,
                    0xc90888f,
                    0x1e230526,
                    0x232bc21,
                    0x1e870526,
                    0xc943d26,
                    0x1e111084,
                    0x19264a4c,
                    0xc978526);
//======

bool getPixel(int data,int x,int y){
  int vtexx=int(tex_map.x*320.0),vtexy=int(tex_map.y*240.0);
  int res=data;
  int coordx,coordy;
  coordx=vtexx-x;
  coordy=vtexy-y;
  if (coordy<=6)
    res>>=coordy*5;
  if (coordx<=5)
    res>>=4-coordx;
  res&=1;
  if (coordx>5 || coordy>6 || coordx<0 || coordy<0)
    return false;
  return res==1;
}
int drawNumber(int num,int x0,int y0){
  int res=num;
  int x=x0;
  while (res!=0){
  //for (int i=0;i<5;i++){
    if (getPixel(number[res%10],x,y0))
      return 1;
    res/=10;
    x-=6;
  }
  return 0;
}
