#include "cd32.h"
#include "cd32_gfx.h"
#include "cd32_pad.h"

static uint16_t frame = 0;

static const int sin_tab[1024] = {
    0,6,12,18,25,31,37,43,49,55,61,67,73,79,85,91,97,102,108,114,119,125,
    130,136,141,146,151,156,161,166,171,176,181,185,190,194,198,203,207,211,
    215,219,222,226,230,233,237,240,243,246,249,252,255,258,260,263,265,267,
    270,272,274,275,277,279,280,282,283,284,285,286,287,288,288,289,289,290,
    290,290,290,290,290,290,289,289,288,288,287,286,285,284,283,282,280,279,
    277,275,274,272,270,267,265,263,260,258,255,252,249,246,243,240,237,233,
    230,226,222,219,215,211,207,203,198,194,190,185,181,176,171,166,161,156,
    151,146,141,136,130,125,119,114,108,102,97,91,85,79,73,67,61,55,49,43,
    37,31,25,18,12,6,0,-6,-12,-18,-25,-31,-37,-43,-49,-55,-61,-67,-73,-79,
    -85,-91,-97,-102,-108,-114,-119,-125,-130,-136,-141,-146,-151,-156,-161,
    -166,-171,-176,-181,-185,-190,-194,-198,-203,-207,-211,-215,-219,-222,
    -226,-230,-233,-237,-240,-243,-246,-249,-252,-255,-258,-260,-263,-265,
    -267,-270,-272,-274,-275,-277,-279,-280,-282,-283,-284,-285,-286,-287,
    -288,-288,-289,-289,-290,-290,-290,-290,-290,-290,-290,-289,-289,-288,
    -288,-287,-286,-285,-284,-283,-282,-280,-279,-277,-275,-274,-272,-270,
    -267,-265,-263,-260,-258,-255,-252,-249,-246,-243,-240,-237,-233,-230,
    -226,-222,-219,-215,-211,-207,-203,-198,-194,-190,-185,-181,-176,-171,
    -166,-161,-156,-151,-146,-141,-136,-130,-125,-119,-114,-108,-102,-97,
    -91,-85,-79,-73,-67,-61,-55,-49,-43,-37,-31,-25,-18,-12,-6
};

static inline int my_sin(int d) { d %= 1024; if (d < 0) d += 1024; return sin_tab[d]; }
static inline int my_cos(int d) { return my_sin(d + 256); }

static uint32_t hsv2rgb(int h) {
    int r, g, b;
    int region = (h % 256) / 43;
    int remainder = ((h % 256) - region * 43) * 6;
    int p = 0, q = 0, t = 0;
    (void)p; (void)q; (void)t;
    switch (region) {
        case 0: r = 255; g = t = remainder * 255 / 255; b = 0; break;
        case 1: r = q = (255 - remainder * 255 / 255); g = 255; b = 0; break;
        case 2: r = 0; g = 255; b = t = remainder * 255 / 255; break;
        case 3: r = 0; g = q = (255 - remainder * 255 / 255); b = 255; break;
        case 4: r = t = remainder * 255 / 255; g = 0; b = 255; break;
        default: r = 255; g = 0; b = q = (255 - remainder * 255 / 255); break;
    }
    return (r << 16) | (g << 8) | b;
}

void game_main(void) {
    cd32_gfx_init();
    cd32_pad_init();
    cd32_audio_init();

    int verts[8][3] = {
        {-50,-50, 50}, { 50,-50, 50}, { 50, 50, 50}, {-50, 50, 50},
        {-50,-50,-50}, { 50,-50,-50}, { 50, 50,-50}, {-50, 50,-50}
    };
    int faces[6][4] = {
        {0,1,2,3}, {1,5,6,2}, {5,4,7,6}, {4,0,3,7}, {3,2,6,7}, {4,5,1,0}
    };

    while (1) {
        cd32_pad_update();
        int rot = frame * 2;

        int sx[8], sy[8];
        for (int i = 0; i < 8; i++) {
            int x3d = verts[i][0], y3d = verts[i][1], z3d = verts[i][2];
            int rx = (my_cos(rot) * x3d - my_sin(rot) * z3d) / 290;
            int rz = (my_sin(rot) * x3d + my_cos(rot) * z3d) / 290;
            int cx = CD32_FB_W / 2, cy = CD32_FB_H / 2;
            int d = rz + 400;
            if (d < 1) d = 1;
            sx[i] = cx + rx * 300 / d;
            sy[i] = cy + y3d * 300 / d;
        }

        cd32_dl_t *dl = cd32_gfx_begin();
        cd32_gfx_clear(dl, 0x102030);

        int hue = (frame * 4) % 256;
        uint32_t colors[6];
        for (int f = 0; f < 6; f++)
            colors[f] = hsv2rgb((hue + f * 43) % 256);

        for (int f = 0; f < 6; f++) {
            cd32_gfx_tri(dl, sx[faces[f][0]], sy[faces[f][0]],
                              sx[faces[f][1]], sy[faces[f][1]],
                              sx[faces[f][2]], sy[faces[f][2]], colors[f]);
            cd32_gfx_tri(dl, sx[faces[f][0]], sy[faces[f][0]],
                              sx[faces[f][2]], sy[faces[f][2]],
                              sx[faces[f][3]], sy[faces[f][3]], colors[f]);
        }

        for (int e = 0; e < 12; e++) {
            static const uint8_t edges[12][2] = {
                {0,1},{1,2},{2,3},{3,0},{4,5},{5,6},{6,7},{7,4},
                {0,4},{1,5},{2,6},{3,7}
            };
            cd32_gfx_line(dl, sx[edges[e][0]], sy[edges[e][0]],
                                sx[edges[e][1]], sy[edges[e][1]], 0xFFFFFF);
        }

        const cd32_pad_state_t *pad = cd32_pad_get(0);
        if (pad->pressed & CD32_JOY_A)
            cd32_audio_play(0, (int16_t[]){5000,-5000,5000,-5000}, 4, 1);
        if (pad->pressed & CD32_JOY_B)
            cd32_audio_play(1, (int16_t[]){-5000,5000,-5000,5000}, 4, 1);

        cd32_gfx_submit(dl);
        cd32_printf("Tech Demo  Frame: %d  Hue: %d\n", frame, hue);

        frame++;
        if (frame > 9999) frame = 0;
    }
}
