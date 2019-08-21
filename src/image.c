/*
 * image.c
 *
 *  Created on: 23 авг. 2017 г.
 *      Author: ivan
 */
/*
#include "image.h"

static uint8_t pack_data[8*8];
static uint8_t unpack_data[8*8];

void IMAGE_New(IMAGE* img,uint16_t width,uint16_t height,PIX_FORMAT format)
{
	img->format = format;
	if (format == RGBI || format == YBRI)
	{
		img->size = width*height*3;
	}
	else if (format == GRAYI)
	{
		img->size = width*height;
	}
	else
	{
		img->format = GRAYI;
		img->size = width*height;
	}
	img->width = width;
	img->height = height;
	img->data = malloc(img->size);
	img->levels_count = 0;
}

void IMAGE_Load_JPEG(IMAGE* img,char* filename)
{
	read_JPEG_file(filename,&img->data,&img->width,&img->height);
	img->format = RGBI;
	img->size = img->width*img->height*3;
	img->levels = calloc(img->width*img->height/(side*side),2);
}

void IMAGE_Get_Rect(IMAGE* img,uint16_t x0,uint16_t y0,uint16_t x1,uint16_t y1,uint8_t* data,uint8_t step)
{
	if (step==0) step = 1;
	uint32_t i=0;
	for (uint16_t y=y0;y<y1;y+=step)
	for (uint16_t x=x0;x<x1;x+=step)
	{
		data[i] = img->data[x + y*img->width];
		i++;
	}
}

void IMAGE_Scaled_Rect(IMAGE* img,uint16_t x0,uint16_t y0,uint16_t x1,uint16_t y1,uint8_t* data)
{
	for (uint16_t y=y0;y<y1;y++)
	for (uint16_t x=x0;x<x1;x++)
	{
		img->data[x + y*img->width] = *data;
		data++;
	}
}

void IMAGE_Fill_Rect(IMAGE* img,uint16_t x0,uint16_t y0,uint16_t x1,uint16_t y1,uint8_t value)
{
	for (uint16_t y=y0;y<y1;y++)
	for (uint16_t x=x0;x<x1;x++)
	{
		img->data[x + y*img->width] = value;
		if (x==x0 && x>1)// && img->data[x-1 + y*img->width]==0)
			img->data[x-1 + y*img->width] = (img->data[x-2 + y*img->width] + img->data[x + y*img->width])/2;
		if (y==y0 && y>1)// && img->data[x + (y-1)*img->width]==0)
			img->data[x + (y-1)*img->width] = (img->data[x + (y-2)*img->width] + img->data[x + y*img->width])/2;
		if (y==y0 && x==x0 && x>1 && y>1)// && img->data[x-1 + (y-1)*img->width]==0)
			img->data[x-1 + (y-1)*img->width] = (img->data[x-2 + (y-2)*img->width] + img->data[x + y*img->width])/2;
	}
}

void IMAGE_Fill_Data(IMAGE* img,uint16_t x0,uint16_t y0,uint16_t x1,uint16_t y1,uint8_t* data)
{
	for (uint16_t y=y0;y<y1;y++)
	{
		for (uint16_t x=x0;x<x1;x++)
		{
			img->data[x + y*img->width] = *data;
			if (x==x0 && x>1)// && img->data[x-1 + y*img->width]==0)
				img->data[x-1 + y*img->width] = (img->data[x-2 + y*img->width] + img->data[x + y*img->width])/2;
			if (y==y0 && y>1)// && img->data[x + (y-1)*img->width]==0)
				img->data[x + (y-1)*img->width] = (img->data[x + (y-2)*img->width] + img->data[x + y*img->width])/2;
			if (y==y0 && x==x0 && x>1 && y>1)// && img->data[x-1 + (y-1)*img->width]==0)
				img->data[x-1 + (y-1)*img->width] = (img->data[x-2 + (y-2)*img->width] + img->data[x + y*img->width])/2;
			data++;
		}
	}
}

void IMAGE_Fill_Scaled_Data(IMAGE* img,uint16_t x0,uint16_t y0,uint16_t x1,uint16_t y1,uint8_t* data)
{
	uint16_t width = (x1-x0)/2;
	uint32_t ind1;
	uint32_t ind2;
	uint32_t ind3;
	uint32_t ind4;
	uint16_t x,y;
	for (y=y0;y<y1;y+=2)
	{
		for (x=x0;x<x1;x+=2)
		{
			ind1 = ((x-x0)/2   + (y-y0)/2*width);
			ind2 = (x-2 + (y)  *img->width);
			ind3 = (x   + (y-2)*img->width);
			ind4 = (x-2 + (y-2)*img->width);
			img->data[x  + y*img->width]	= data[ind1];
			if (x>1)// && img->data[x-1+ y*img->width]==0)
				img->data[x-1+ y*img->width]	= (data[ind1] + img->data[ind2])/2;
			if (y>1)// && img->data[x  + (y-1)*img->width]==0)
				img->data[x  + (y-1)*img->width]= (data[ind1] + img->data[ind3])/2;
			if (y>1 && x>1)// && img->data[x-1+ (y-1)*img->width]==0)
				img->data[x-1+ (y-1)*img->width]= (data[ind1] + img->data[ind2] + img->data[ind3] + img->data[ind4])/4;
		}
	}
}

uint8_t _get_2bit(uint8_t* arr,size_a index)
{
	return (arr[index/4]>>(index%4*2))&3;
}
uint8_t _get_4bit(uint8_t* arr,size_a index)
{
	return (index%2) ? (arr[index/2]>>4) : (arr[index/2]&15);
}

void _set_2bit(uint8_t* arr,size_a index,uint8_t val)
{
	arr[index/4] = (arr[index/4] & ~(3<<(index%4*2))) | ((val&3)<<(index%4*2));
}
void _set_4bit(uint8_t* arr,size_a index,uint8_t val)
{
	uint8_t temp = arr[index/2];
	temp &= index%2 ? 0x0F : 0xF0;
	temp |= index%2 ? val<<4 : val&15;
	arr[index/2] = temp;
}

void IMAGE_Merge(IMAGE* img,IMAGE* ch1,IMAGE* ch2,IMAGE* ch3)
{
	if (ch1->format!=GRAYI || ch2->format!=GRAYI || ch3->format!=GRAYI ||
			ch1->size*3!=img->size || ch2->size*3!=img->size || ch3->size*3!=img->size)
	{
		printf("Error\n");
		exit(1);
		return;
	}
	for (uint32_t i=0;i<img->size;i+=3)
	{
		img->data[i]   = ch1->data[i/3];
		img->data[i+1] = ch2->data[i/3];
		img->data[i+2] = ch3->data[i/3];
	}
}

void _decode_8x8(IMAGE* img,FILE* fp,uint16_t x0,uint16_t y0,uint16_t x1,uint16_t y1)
{
	uint8_t cmd,cmd2;
	size_a res;
	res = fread(&cmd,1,1,fp);
	res = fread(&cmd2,1,1,fp);
	if ((cmd&0x3) == 0)
	{
		if (cmd2 > 0)
		{
			//IMAGE_Fill_Rect(img,x0,y0,x1-1,y1-1,0);
			IMAGE_Fill_Rect(img,x0,y0,x1,y1,cmd2);
		}
		else
		{
			//IMAGE_Fill_Rect(img,x0,y0,x1,y1,0);
		}
	}
	else if ((cmd&0x3) == 1)
	{
		res = fread(pack_data,8*8/16,1,fp);
		for (uint16_t i=0;i<8*8/4;i++)
		{
			unpack_data[i] = _get_2bit(pack_data,i);
			unpack_data[i] = clamp(0,255,unpack_data[i]*cmd2/3.0 + ((cmd&0xFC) | 2));
		}
		//IMAGE_Fill_Rect(img,x0-1,y0-1,x1-1,y1-1,0);
		IMAGE_Fill_Scaled_Data(img,x0,y0,x1,y1,unpack_data);
	}
	else if ((cmd&0x3) == 2)
	{
		res = fread(pack_data,8*8/4,1,fp);
		for (uint16_t i=0;i<8*8;i++)
		{
			unpack_data[i] = _get_2bit(pack_data,i);
			unpack_data[i] = clamp(0,255,unpack_data[i]/3.0*cmd2 + cmd);
		}
		//IMAGE_Fill_Rect(img,x0,y0,x1-1,y1-1,0);
		IMAGE_Fill_Data(img,x0,y0,x1,y1,unpack_data);
	}
	else
	{
		res = fread(pack_data,8*8/2,1,fp);
		for (uint16_t i=0;i<8*8;i++)
		{
			unpack_data[i] = _get_4bit(pack_data,i);
			unpack_data[i] = clamp(0,255,unpack_data[i]/15.0*cmd2 + cmd);
		}
		//IMAGE_Fill_Rect(img,x0,y0,x1-1,y1-1,0);
		IMAGE_Fill_Data(img,x0,y0,x1,y1,unpack_data);
	}
}

void _encode_8x8(IMAGE* img,FILE* fp,uint16_t x0,uint16_t y0,uint16_t x1,uint16_t y1)
{
	uint8_t max=0,min=255,pix;
	uint8_t maxd=0,mind=255,pixd;
	uint16_t width = x1-x0;
	uint16_t height = y1-y0;
	float aDiff = 0.0;
	for (uint32_t y=y0;y<y1;y++)
	for (uint32_t x=x0;x<x1;x++)
	{
		pix = img->data[x + y*img->width];
		if (x1-x>1 && y1-y>1)
			pixd = (abs(img->data[x + y*img->width]-img->data[x+1 + y*img->width]) + abs(img->data[x + y*img->width]-img->data[x + (y+1)*img->width])) / 2.0;
		else
			pixd = 0;
		max = pix>max ? pix : max;
		min = pix<min ? pix : min;
		maxd = pixd>maxd ? pixd : maxd;
		mind = pixd<mind ? pixd : mind;
		aDiff += pixd/width/height;
	}
	if (max==min)
	{
		//printf("Monochrome block\n");
		pix = 0;
		fwrite(&pix,1,1,fp);
		min += min==0;
		fwrite(&min,1,1,fp);
	}
	else if (maxd<15)
	{
		//printf("HalfRes block\n");
		pix = (min&0xFC) | 1;
		fwrite(&pix,1,1,fp);
		pix = max-min;
		fwrite(&pix,1,1,fp);
		IMAGE_Get_Rect(img,x0,y0,x1,y1,unpack_data,2);
		for (uint16_t i=0;i<8*8;i++)
		{
			unpack_data[i] = (unpack_data[i]-min)/(float)(max-min)*3.0 + 0.49;
			_set_2bit(pack_data,i,unpack_data[i]);
		}
		fwrite(pack_data,8*8/16,1,fp);
	}
	else// if (abs(aDiff-(maxd+mind)/2) < 30)//aDiff<(maxd+mind)/2.0)
	{
		//printf("NormRes block\n");
		pix = (min&0xFC) | 2;
		fwrite(&pix,1,1,fp);
		pix = max-min;
		fwrite(&pix,1,1,fp);
		IMAGE_Get_Rect(img,x0,y0,x1,y1,unpack_data,1);
		for (uint16_t i=0;i<8*8;i++)
		{
			unpack_data[i] = (unpack_data[i]-min)/(float)(max-min)*3.0 + 0.49;
			_set_2bit(pack_data,i,unpack_data[i]);
		}
		fwrite(pack_data,8*8/4,1,fp);
	}
}

void IMAGE_Save_STUP(IMAGE* img,char* filename)
{
	FILE* fp = fopen(filename,"wb");

	fwrite(&img->width,2,1,fp);
	fwrite(&img->height,2,1,fp);

	IMAGE_Convert(img,YBRI);
	IMAGE* ch = IMAGE_Split(img);

	//IMAGE_Selective_Blur(ch,2,4,50);

	IMAGE_Scale(&ch[1],1);
	IMAGE_Scale(&ch[2],1);
	uint16_t x=0,y=0;

	for (uint16_t i=0;i<img->width*img->height/8/8;i+=4)
	{
		_encode_8x8(&ch[0],fp,x,y,x+8,y+8);
		_encode_8x8(&ch[0],fp,x+8,y,x+16,y+8);
		_encode_8x8(&ch[0],fp,x,y+8,x+8,y+16);
		_encode_8x8(&ch[0],fp,x+8,y+8,x+16,y+16);

		_encode_8x8(&ch[1],fp,x/2,y/2,x/2+8,y/2+8);
		_encode_8x8(&ch[2],fp,x/2,y/2,x/2+8,y/2+8);

		x += 16;
		if (x>=img->width)
		{
			x = 0;
			y += 16;
		}
	}

	IMAGE_Free(&ch[0]);
	IMAGE_Free(&ch[1]);
	IMAGE_Free(&ch[2]);
	free(ch);
	fclose(fp);
}

void IMAGE_Overlay(IMAGE* img,IMAGE* img2)
{
	if (img->size!=img2->size) return;
	for (uint32_t i=0;i<img->size;i++)
	{
		img->data[i] = img2->data[i] ? img2->data[i] : img->data[i];
	}
}

void IMAGE_Load_STUP(IMAGE* img,char* filename)
{
	size_a res = 0;
	FILE* fp = fopen(filename,"rb");
	if (fp==0)
	{
		puts(filename);
		printf(" not found\n");
		exit(1);
	}
	uint16_t resolution[2];
	res = fread(resolution,2,2,fp);
	IMAGE value,cb,cr;
	//printf("Image %dx%d\n",resolution[0],resolution[1]);
	IMAGE_New(&value,resolution[0],resolution[1],GRAYI);
	IMAGE_New(&cb,resolution[0]/2,resolution[1]/2,GRAYI);
	IMAGE_New(&cr,resolution[0]/2,resolution[1]/2,GRAYI);
	IMAGE_New(img,resolution[0],resolution[1],YBRI);
	uint16_t x=0,y=0;
	for (uint16_t i=0;i<resolution[0]*resolution[1]/8/8;i+=4)
	{
		_decode_8x8(&value,fp,x,y,x+8,y+8);
		_decode_8x8(&value,fp,x+8,y,x+16,y+8);
		_decode_8x8(&value,fp,x,y+8,x+8,y+16);
		_decode_8x8(&value,fp,x+8,y+8,x+16,y+16);

		_decode_8x8(&cb,fp,x/2,y/2,x/2+8,y/2+8);
		_decode_8x8(&cr,fp,x/2,y/2,x/2+8,y/2+8);

		x += 16;
		if (x>=img->width)
		{
			x = 0;
			y += 16;
		}
	}
	IMAGE_Scale(&cb,0);
	IMAGE_Scale(&cr,0);
	//IMAGE_Selective_Blur(&value,1,8,10);
	IMAGE_Merge(img,&value,&cb,&cr);
	IMAGE_Free(&value);
	IMAGE_Free(&cb);
	IMAGE_Free(&cr);
	fclose(fp);
}

void IMAGE_Load_STUP2(IMAGE* value,IMAGE* cb,IMAGE* cr,char* filename)
{
	size_a res = 0;
	FILE* fp = fopen(filename,"rb");
	if (fp==0)
	{
		puts(filename);
		printf(" not found\n");
		exit(1);
	}
	uint16_t resolution[2];
	res = fread(resolution,2,2,fp);
	uint16_t x=0,y=0;
	for (uint16_t i=0;i<resolution[0]*resolution[1]/8/8;i+=4)
	{
		_decode_8x8(value,fp,x,y,x+8,y+8);
		_decode_8x8(value,fp,x+8,y,x+16,y+8);
		_decode_8x8(value,fp,x,y+8,x+8,y+16);
		_decode_8x8(value,fp,x+8,y+8,x+16,y+16);

		_decode_8x8(cb,fp,x/2,y/2,x/2+8,y/2+8);
		_decode_8x8(cr,fp,x/2,y/2,x/2+8,y/2+8);

		x += 16;
		if (x>=value->width)
		{
			x = 0;
			y += 16;
		}
	}
	fclose(fp);
}

void IMAGE_Save_JPEG(IMAGE* img,char* name)
{
	write_JPEG_file (name,img->data,img->width,img->height, img->format, 100);
}
void IMAGE_Stream_JPEG(IMAGE* img,FILE* stream)
{
	write_JPEG_stream (stream,img->data,img->width,img->height, img->format, 85);
}

void RGB2YBR(uint8_t* pixel)
{
	float Kr,Kb,Kg;
	Kr = 0.299;
	Kb = 0.114;
	Kg = 1.0 - Kr - Kb;
	uint8_t value = (pixel[0]*Kr + pixel[1]*Kg + pixel[2]*Kb);
	uint8_t Cblue = 128.0 + 0.5 * (pixel[2] - value) / (1.0 - Kb);
	uint8_t Cred  = 128.0 + 0.5 * (pixel[0] - value) / (1.0 - Kr);
	pixel[0] = value;
	pixel[1] = Cblue;
	pixel[2] = Cred;
}

void YBR2RGB(uint8_t* pixel)
{
	float red	= 298.082*pixel[0]/256.0 + 408.583*pixel[2]/256.0 - 222.921;
	float green	= 298.082*pixel[0]/256.0 - 100.291*pixel[1]/256.0 - 208.12*pixel[2]/256.0 + 135.576;
	float blue	= 298.082*pixel[0]/256.0 + 512.412*pixel[1]/256.0 - 276.836;
	red = clamp(0.0,255.0,red*0.9372549019607843+16);
	green = clamp(0.0,255.0,green*0.9372549019607843+16);
	blue = clamp(0.0,255.0,blue*0.9372549019607843+16);
	pixel[0] = red;
	pixel[1] = green;
	pixel[2] = blue;
}

void IMAGE_Convert(IMAGE* img,PIX_FORMAT format)
{
	if (img->format == format) return;
	//printf("Converting ");
	uint8_t* new_data;
	if (img->format == GRAYI)
	{
		new_data = malloc(img->size*3);
		img->size *= 3;
		if (format==RGBI)
		{
			//printf("Grayscale to RGB\n");
			for (uint32_t i=0;i<img->size;i+=3)
			{
				new_data[i+0] = img->data[i];
				new_data[i+1] = img->data[i];
				new_data[i+2] = img->data[i];
			}
		}
		else
		{
			for (uint32_t i=0;i<img->size;i+=3)
			{
				//printf("Grayscale to YCbCr\n");
				new_data[i+0] = img->data[i];
				new_data[i+1] = 128;
				new_data[i+2] = 128;
			}
		}
		free(img->data);
		img->data = new_data;
	}
	else if (img->format == RGBI)
	{
		if (format == YBRI)
		{
			//printf("RGB to YCbCr\n");
			for (uint32_t i=0;i<img->size;i+=3)
			{
				RGB2YBR(&img->data[i]);
			}
		}
		else
		{
			//printf("RGB to grayscale\n");
			new_data = calloc(img->size/3,1);
			for (uint32_t i=0;i<img->size;i+=3)
			{
				new_data[i/3] = (img->data[i] + img->data[i+1] + img->data[i+2]) / 3.0 + 0.5;
			}
			img->size /= 3;
			free(img->data);
			img->data = new_data;
		}
	}
	else if (img->format == YBRI)
	{
		if (format == RGBI)
		{
			//printf("YCbCr to RGB\n");
			for (uint32_t i=0;i<img->size;i+=3)
			{
				YBR2RGB(&img->data[i]);
			}
		}
		else
		{
			//printf("YCbCr to grayscale\n");
			for (uint32_t i=0;i<img->size;i+=3)
			{
				img->data[i/3] = img->data[i];
			}
			img->size /= 3;
			//printf("Reallocating\n");
			//img->data = realloc(img->data,img->size);
		}
	}
	img->format = format;
	//printf("Converted\n");
}

IMAGE* IMAGE_Split(IMAGE* img)
{
	if (img->format != RGBI && img->format != YBRI) return (IMAGE*)0;

	IMAGE* ch = calloc(sizeof(IMAGE),3);
	IMAGE_New(&ch[0],img->width,img->height,GRAYI);
	IMAGE_New(&ch[1],img->width,img->height,GRAYI);
	IMAGE_New(&ch[2],img->width,img->height,GRAYI);

	for (uint32_t i=0;i<img->width*img->height*3;i+=3)
	{
		((uint8_t*)(ch[0].data))[i/3] = ((uint8_t*)img->data)[i];
		((uint8_t*)(ch[1].data))[i/3] = ((uint8_t*)img->data)[i+1];
		((uint8_t*)(ch[2].data))[i/3] = ((uint8_t*)img->data)[i+2];
	}

	return ch;
}

void IMAGE_Scale(IMAGE* img,_Bool scale)
{
	uint8_t* new_data;
	if (scale)
	{
		if (img->format == GRAYI)
		{
			new_data = malloc(img->width*img->height/4);
			for (uint16_t y=0;y<img->height-1;y+=2)
			{
				for (uint16_t x=0;x<img->width-1;x+=2)
				{
					uint8_t pix1 = img->data[y*img->width + x];
					uint8_t pix2 = img->data[(y)*img->width + x+1];
					uint8_t pix3 = img->data[(y+1)*img->width + x];
					uint8_t pix4 = img->data[(y+1)*img->width + x+1];
					new_data[x/2 + y/2*img->width/2] = (pix1+pix2+pix3+pix4)/4.0+0.4;
				}
			}
			free(img->data);
			img->data = new_data;
			img->size/=4;
			img->width/=2;
			img->height/=2;
		}
	}
	else
	{
		if (img->format == GRAYI)
		{
			new_data = malloc(img->width*img->height*5);

			for (uint16_t y=0;y<img->height;y++)
			{
				for (uint16_t x=0;x<img->width;x++)
				{
					uint8_t pix1 = img->data[x   + y*img->width];
					uint8_t pix2 = img->data[x+1 + (y)*img->width];
					uint8_t pix3 = img->data[x   + (y+1)*img->width];
					uint8_t pix4 = img->data[x+1 + (y+1)*img->width];
					new_data[x*2   + (y*2)  *img->width*2] = pix1;
					new_data[x*2+1 + (y*2)  *img->width*2] = (pix1+pix2)/2.0 + 0.4;
					new_data[x*2   + (y*2+1)*img->width*2] = (pix1+pix3)/2.0 + 0.4;
					new_data[x*2+1 + (y*2+1)*img->width*2] = (pix1+pix2+pix3+pix4)/4.0 + 0.4;
				}
			}
			free(img->data);
			img->data = new_data;
			img->width*=2;
			img->height*=2;
			img->size*=4;
		}
	}
}

void IMAGE_Scaled(IMAGE* img,IMAGE* img2,_Bool scale)
{
	if (scale)
	{
		if (img->format == GRAYI)
		{
			for (uint16_t y=0;y<img->height-1;y+=2)
			{
				for (uint16_t x=0;x<img->width-1;x+=2)
				{
					uint8_t pix1 = img->data[y*img->width + x];
					uint8_t pix2 = img->data[(y)*img->width + x+1];
					uint8_t pix3 = img->data[(y+1)*img->width + x];
					uint8_t pix4 = img->data[(y+1)*img->width + x+1];
					img2->data[x/2 + y/2*img->width/2] = (pix1+pix2+pix3+pix4)/4.0+0.4;
				}
			}
		}
	}
	else
	{
		if (img->format == GRAYI)
		{

			for (uint16_t y=0;y<img->height;y++)
			{
				for (uint16_t x=0;x<img->width;x++)
				{
					uint8_t pix1 = img->data[x   + y*img->width];
					uint8_t pix2 = img->data[x+1 + (y)*img->width];
					uint8_t pix3 = img->data[x   + (y+1)*img->width];
					uint8_t pix4 = img->data[x+1 + (y+1)*img->width];
					img2->data[x*2   + (y*2)  *img->width*2] = pix1;
					img2->data[x*2+1 + (y*2)  *img->width*2] = (pix1+pix2)/2.0 + 0.4;
					img2->data[x*2   + (y*2+1)*img->width*2] = (pix1+pix3)/2.0 + 0.4;
					img2->data[x*2+1 + (y*2+1)*img->width*2] = (pix1+pix2+pix3+pix4)/4.0 + 0.4;
				}
			}
		}
	}
}

void IMAGE_Free(IMAGE* img)
{
	if (img->size == 0) return;
	img->width = img->height = img->size = 0;
	free(img->data);
	//free(img->levels);
}

void IMAGE_Blur(IMAGE* img,uint8_t radius)
{
	uint8_t* horizontal = (uint8_t*)malloc(img->size);
	if (img->format == GRAYI)
	{
		for (uint16_t y=0;y<img->height;y++)
		for (uint16_t x=radius;x<img->width-radius;x++)
		{
			float val = 0.0;
			for (int8_t i=-radius;i<=radius;i++)
				val += img->data[x+i + y*img->width];
			horizontal[x + y*img->width] = val/(radius*2+1);
		}
		for (uint16_t y=radius/2;y<img->height-radius/2;y++)
		for (uint16_t x=0;x<img->width;x++)
		{
			float val = 0.0;
			for (int8_t i=-radius;i<=radius;i++)
				val += horizontal[x + (y+i)*img->width];
			img->data[x + y*img->width] = val/(radius*2+1);
		}
	}
	//printf("Blurred\n");
	free(horizontal);
}

void IMAGE_Blur2(IMAGE* img,uint8_t radius,uint8_t threshold)
{
	uint8_t* horizontal = (uint8_t*)calloc(img->size,1);
	if (img->format == GRAYI)
	{
		for (uint16_t y=0;y<img->height;y++)
		for (uint16_t x=radius;x<img->width-radius;x++)
		{
			for (int8_t i=-radius;i<=radius;i++)
				horizontal[x + y*img->width] = img->data[x+i + y*img->width]>horizontal[x + y*img->width] && img->data[x+i + y*img->width]>threshold
																						? img->data[x+i + y*img->width] : horizontal[x + y*img->width];
		}
		for (uint16_t y=radius/2;y<img->height-radius/2;y++)
		for (uint16_t x=0;x<img->width;x++)
		{
			for (int8_t i=-radius;i<=radius;i++)
				img->data[x + y*img->width] = horizontal[x + (y+i)*img->width]>img->data[x + y*img->width] && horizontal[x + (y+i)*img->width]>threshold
																						 ? horizontal[x + (y+i)*img->width] : img->data[x + y*img->width];
		}
	}
	//printf("Blurred\n");
	free(horizontal);
}

float _fclamp(float a,float b, float val)
{
	if (val<a)
		return a;
	else if (val>b)
		return b;
	else
		return val;
}

void IMAGE_CB(IMAGE* img,float contrast,int16_t bright)
{
	for (uint32_t i=0;i<img->size;i++)
	{

		img->data[i] = _fclamp(0.0,255.0,(float)(img->data[i]-128.0)*contrast + bright + 128.5);
	}
}

void IMAGE_Threshold(IMAGE* img,int8_t val)
{
	for (uint32_t i=0;i<img->size;i++)
	{

		img->data[i] = img->data[i]>val ? 255 : 0;
	}
}

void IMAGE_Delta(IMAGE* img)
{
	int32_t val,delta;
	for (uint32_t y=1;y<img->height;y++)
	{
		val = 0;
		for (uint32_t x=1;x<img->width;x++)
		{
			delta = (val - img->data[x + y*img->width]);
			val = img->data[x + y*img->width];
			img->data[x + y*img->width] = delta;
		}
	}
}

void IMAGE_Delta_Dec(IMAGE* img)
{
	int32_t val;
	for (uint32_t y=1;y<img->height;y++)
	{
		val = 0;
		for (uint32_t x=1;x<img->width;x++)
		{
			val -= (char)img->data[x + y*img->width];
			img->data[x + y*img->width] = _fclamp(0,255,val);
		}
	}
}

void IMAGE_Invert(IMAGE* img)
{
	for (uint32_t i=0;i<img->size;i++)
	{
		img->data[i] = 255 - img->data[i];
	}
}

void IMAGE_Edge_Filter(IMAGE* img)
{
	uint8_t* edge = malloc(img->size);
	for (uint16_t y=1;y<img->height;y++)
	for (uint16_t x=1;x<img->width;x++)
	{
		uint8_t pix0 = img->data[x   +  y*img->width];
		uint8_t pix1 = img->data[x-1 +  y*img->width];
		uint8_t pix2 = img->data[x   + (y-1)*img->width];
		int val = sqrt(abs(pix0-pix1)*abs(pix0-pix1) + abs(pix0-pix2)*abs(pix0-pix2));
		edge[x   +  y*img->width] = clamp(0,255,val);
	}
	free(img->data);
	img->data = edge;
}

void IMAGE_Copy(IMAGE* img,IMAGE* new_img)
{
	IMAGE_New(new_img,img->width,img->height,img->format);
	for (uint32_t i=0;i<img->size;i++)
	{
		new_img->data[i] = img->data[i];
	}
}

void IMAGE_Selective_Blur(IMAGE* img,float bradius,float thr,float thr2)
{
	IMAGE blur_map;
	IMAGE_Copy(img,&blur_map);
	IMAGE_Edge_Filter(&blur_map);
	IMAGE_Blur2(&blur_map,bradius,5);
	IMAGE_Blur(&blur_map,bradius);
	IMAGE_CB(&blur_map,thr,thr2*thr);
	//IMAGE_Save_JPEG(&blur_map,"/home/ivan/bm.jpg");
	IMAGE_Invert(&blur_map);
	IMAGE_CB(&blur_map,bradius/255.0,-127);

	uint8_t* new_data = malloc(img->width*img->height);
	for (uint16_t y=0;y<img->height;y++)
	for (uint16_t x=0;x<img->width;x++)
	{
		double val = img->data[x + (y)*img->width];
		uint8_t radius = blur_map.data[x + (y)*img->width];
		double vals = 1;

		for (int16_t j=-radius;j<=radius;j++)
		for (int16_t i=-radius;i<=radius;i++)
		{
			if (x+i<blur_map.width && y+j>0 && y+j<blur_map.height)
			{
				val += (float)img->data[x+i + (y+j)*img->width]*blur_map.data[x+i + (y+j)*blur_map.width]/bradius;
				vals+=blur_map.data[x+i + (y+j)*blur_map.width]/bradius;
			}
		}
		val /= vals;
		new_data[x + y*img->width] = val;
	}
	free(img->data);
	img->data = new_data;
}
*/
