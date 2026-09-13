#pragma once

#include "quantum.h"

#ifdef RAW_ENABLE

#define KLH_CMD_LAYER_QUERY 0x42
#define KLH_CMD_LAYER_REPORT 0x43

void klh_layer_report_send(layer_state_t layer);
void klh_raw_hid_receive(uint8_t *data, uint8_t length);

#endif
