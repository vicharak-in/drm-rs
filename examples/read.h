/* Copyright (C) VAAMAN 2022 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>

#define CHAR_SIZE 921600

char *fpga_read();

struct fpga_frame {
	int sof;
	int pkt_id;
	int dtype;
	int dlen;
	int phl_id;
	int reserved;
	char* data;
	int eof;
};
