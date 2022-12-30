/* Copyright (C) VAAMAN 2022 */

#include "read.h"

char* exec(char* command) {
	// Open pipe to file
	FILE* pipe = popen(command, "r");

	if (!pipe) return "popen failed!";

	// read till end of process:
	char* line = NULL;
	size_t linecap = 0;

	if (getline(&line, &linecap, pipe) <= 0) printf("Error Occured");

	pclose(pipe);
	return line;
}

void xprintf(char* data) {
	int i = 0;
	char c = data[i];
	
	while (c != EOF) {
		printf("%c", c);
		c = data[++i];
	}
}

int check_frame(struct fpga_frame* f) {
	if (f->sof != 0xeaff ||
			f->dlen != (int)strlen(f->data) ||
			f->eof != 0xaadd)
		return -1;
	else
		return 0;
}

struct fpga_frame* create_frame(char* data) {
	int i;
	char* temp;
	int dlen = ((data[7] << 24) + (data[8] << 16) + (data[9] << 8) + data[10]);

	temp = (char*)malloc(sizeof(char) * dlen);

	for (i = 0; i < dlen; i++)
		temp[i] = data[i + 12];

	struct fpga_frame *f = (struct fpga_frame*)malloc(sizeof(struct fpga_frame));
	f->data = (char*)malloc(sizeof(char) * dlen);

	f->sof = (data[0] << 8) + data[1];
	f->pkt_id = ((data[2] << 24) + (data[3] << 16) + (data[4] << 8) + data[5]);
	f->dtype = data[6];
	f->dlen = dlen;
	f->phl_id = data[11];
	//f->reserved = data[11],
	strcpy(f->data, temp);
	f->eof = (data[dlen + 12] << 8) + data[dlen + 1 + 12];

/*
 *    printf("Frame: {\n\tsof: 0x%x\n\tpkt_id: 0x%x\n\tdtype: 0x%x\n\tdlen: 0x%x\n\tphl_id: "
 *            "0x%x\n\treserved: 0x%x\n\tdata: %s\n\teof: 0x%x\n}\n",
 *            f->sof, f->pkt_id, f->dtype, f->dlen, f->phl_id, f->reserved, f->data, f->eof);
 *
 *    printf("check: %d\n", check_frame(f));
 */

	free(temp);
	/*
	 *free(f);
	 */
	return f;
}

char* fpga_read() {
	/*while (1) {*/
	char *data = exec("v4l2-ctl --stream-mmap --stream-count=1 -d0 --stream-to=/dev/stdout");
	//xprintf(data);

	struct fpga_frame* f = create_frame(data);
	char *fdata = (char*)malloc(sizeof(char) * f->dlen);
	strcpy(fdata, f->data);
	free(f);
	return fdata;
	/*}*/
}
