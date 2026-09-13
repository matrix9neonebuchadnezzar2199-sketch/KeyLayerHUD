#include "layer_report.h"

#ifdef RAW_ENABLE

#include "raw_hid.h"

void klh_layer_report_send(layer_state_t layer) {
    uint8_t report[RAW_EPSIZE] = {0};
    uint32_t mask = layer;

    report[0] = KLH_CMD_LAYER_REPORT;
    report[1] = get_highest_layer(layer);
    report[2] = (uint8_t)(mask & 0xFF);
    report[3] = (uint8_t)((mask >> 8) & 0xFF);
    report[4] = (uint8_t)((mask >> 16) & 0xFF);
    report[5] = (uint8_t)((mask >> 24) & 0xFF);
    report[6] = get_mods();

    raw_hid_send(report, sizeof(report));
}

void klh_raw_hid_receive(uint8_t *data, uint8_t length) {
    if (length < 1) {
        return;
    }
    if (data[0] == KLH_CMD_LAYER_QUERY) {
        klh_layer_report_send(layer_state);
    }
}

layer_state_t layer_state_set_user(layer_state_t state) {
    klh_layer_report_send(state);
    return state;
}

#endif
