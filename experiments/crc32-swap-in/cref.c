#include <stdio.h>
#include <stdlib.h>
#include <time.h>
static unsigned int T[256];
static void mk(void){for(unsigned int n=0;n<256;n++){unsigned int c=n;for(int k=0;k<8;k++)c=(c&1)?(0xEDB88320u^(c>>1)):(c>>1);T[n]=c;}}
static unsigned int crcC(unsigned int crc,const unsigned char*b,unsigned int n){unsigned int c=crc^0xFFFFFFFFu;for(unsigned int i=0;i<n;i++)c=T[(c^b[i])&0xff]^(c>>8);return c^0xFFFFFFFFu;}
static double now(void){struct timespec t;clock_gettime(CLOCK_MONOTONIC,&t);return t.tv_sec+t.tv_nsec*1e-9;}
int main(void){mk();unsigned int n=64u<<20;unsigned char*b=malloc(n);for(unsigned int i=0;i<n;i++)b[i]=(unsigned char)(i*2654435761u>>24);unsigned long s=0;s^=crcC(0,b,n);double t0=now();for(int r=0;r<8;r++)s^=crcC(0,b,n);double t=(now()-t0)/8;printf("scalar-C table crc32: %.2f GB/s (sink %lx)\n",n/t/1e9,s);return 0;}
