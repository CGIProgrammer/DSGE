/*
 * jpeg_example.c
 *
 *  Created on: 15 авг. 2017 г.
 *      Author: ivan
 */
/*#include "jpeg_magic.h"
#include <stdlib.h>

void write_JPEG_stream (FILE * stream,JSAMPLE* img_buffer,uint16_t img_width,uint16_t img_height, uint8_t format, int quality)
{
  struct jpeg_compress_struct cinfo;
  struct jpeg_error_mgr jerr;

  JSAMPROW row_pointer[1];
  int row_stride;

  cinfo.err = jpeg_std_error(&jerr);

  jpeg_create_compress(&cinfo);

  if (stream == NULL)
  {
	fprintf(stderr,"Null stream\n");
    exit(1);
  }
  jpeg_stdio_dest(&cinfo, stream);

  cinfo.image_width = img_width;
  cinfo.image_height = img_height;
  if (format==2)
  {
	  //printf("Saving grayscale JPEG");
	  cinfo.input_components = 1;
	  cinfo.in_color_space = JCS_GRAYSCALE;
  }
  else if (format == 1)
  {
	  //printf("Saving YCbCr JPEG");
	  cinfo.input_components = 3;
	  cinfo.in_color_space = JCS_YCbCr;
  }
  else
  {
	  //printf("Saving RGB JPEG");
	  cinfo.input_components = 3;
	  cinfo.in_color_space = JCS_RGB;
  }
  cinfo.do_fancy_downsampling = 1;
  cinfo.progressive_mode = 0;

  jpeg_set_defaults(&cinfo);
  cinfo.comp_info[0].v_samp_factor = 1;
  cinfo.comp_info[0].h_samp_factor = 1;
  cinfo.comp_info[1].v_samp_factor = 1;
  cinfo.comp_info[1].h_samp_factor = 1;
  cinfo.comp_info[2].v_samp_factor = 1;
  cinfo.comp_info[2].h_samp_factor = 1;
  jpeg_set_quality(&cinfo, quality, TRUE );
  jpeg_start_compress(&cinfo, TRUE);

  if (format == 0 || format == 1)
	  row_stride = img_width * 3;
  else
	  row_stride = img_width;

  while (cinfo.next_scanline < cinfo.image_height)
  {
    row_pointer[0] = & img_buffer[cinfo.next_scanline * row_stride];
    (void) jpeg_write_scanlines(&cinfo, row_pointer, 1);
  }

  jpeg_finish_compress(&cinfo);
  jpeg_destroy_compress(&cinfo);
  //printf(" successful\n");
}

void write_JPEG_file (char * filename,JSAMPLE* img_buffer,uint16_t img_width,uint16_t img_height, uint8_t format, int quality)
{
	FILE *outfile;
  if ((outfile = fopen(filename, "wb")) == NULL)
  {
	printf("can't open %s\n", filename);
    exit(1);
  }
  write_JPEG_stream(outfile,img_buffer,img_width,img_height,format,quality);
}

struct my_error_mgr {
  struct jpeg_error_mgr pub;

  jmp_buf setjmp_buffer;
};

typedef struct my_error_mgr * my_error_ptr;

METHODDEF(void)
my_error_exit (j_common_ptr cinfo)
{
  my_error_ptr myerr = (my_error_ptr) cinfo->err;

  (*cinfo->err->output_message) (cinfo);
  longjmp(myerr->setjmp_buffer, 1);
}

int read_JPEG_file (char * filename, void** ptr, uint16_t* width, uint16_t* height)
{
  struct jpeg_decompress_struct cinfo;
  struct my_error_mgr jerr;
  FILE * infile;
  JSAMPARRAY buffer;
  int row_stride;

  if ((infile = fopen(filename, "rb")) == NULL) {
    fprintf(stderr, "can't open %s\n", filename);
    return 0;
  }

  cinfo.err = jpeg_std_error(&jerr.pub);
  jerr.pub.error_exit = my_error_exit;
  if (setjmp(jerr.setjmp_buffer)) {
    jpeg_destroy_decompress(&cinfo);
    fclose(infile);
    return 0;
  }
  jpeg_create_decompress(&cinfo);

  jpeg_stdio_src(&cinfo, infile);
  (void) jpeg_read_header(&cinfo, TRUE);
  (void) jpeg_start_decompress(&cinfo);
  row_stride = cinfo.output_width * cinfo.output_components;
  buffer = (*cinfo.mem->alloc_sarray)
		((j_common_ptr) &cinfo, JPOOL_IMAGE, row_stride, 1);
  *width = cinfo.output_width;
  *height = cinfo.output_height;
  *ptr = malloc(cinfo.output_width*cinfo.output_height*3);
  char* data = (char*)*ptr;
  while (cinfo.output_scanline < cinfo.output_height) {
    (void) jpeg_read_scanlines(&cinfo, buffer, 1);
    memcpy(data,buffer[0],row_stride);
    data+=row_stride;
  }

  (void) jpeg_finish_decompress(&cinfo);
  jpeg_destroy_decompress(&cinfo);
  fclose(infile);
  return 1;
}*/
