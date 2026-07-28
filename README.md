# Amiga FlameSnow

**Amiga FlameSnow** — Sucessor espiritual do Amiga CD³². Plataforma aberta de
videogame com emulador cycle-accurate, runtime C para jogos (`libcd32.a`),
SDK completo e ferramentas de build.

```text
 Amiga CD³² (1993) → Amiga FlameSnow (1999)
     32 bits               64 bits
     Motorola 68EC020      PowerPC 603e + ColdFire V4e
     Akiko chip            GPU Lisa II TBDR
     2MB Chip RAM          28MB RAM unificada
```

**CI:** `make ci` — pipeline completo, sem dependência externa.

---

## Funcionalidades

| Componente | Status |
|------------|--------|
| Emulador PPC603e cycle-accurate + MMU | ✅ |
| Coprocessador ColdFire V4e (companion) | ✅ |
| GPU TBDR "Lisa II" (tile-based, 640×480) | ✅ |
| DSP áudio 8 canais estéreo (volume, pan, loop) | ✅ |
| DMA 4 canais (CDROM → GPU → Audio → CF) | ✅ |
| CD-ROM 12x + ISO9660 parser + ELF loader | ✅ |
| **DVD-ROM opcional** (`mkcd.sh --media dvd`) | ✅ |
| **Memory Card** (2 slots × 512KB, persistente) | ✅ |
| **Controle analógico** (4 eixos, game controller SDL) | ✅ |
| Save states (compressão, 9 seções) | ✅ |
| Disassembler PPC + ColdFire (`--trace`) | ✅ |
| Interrupt controller (8 níveis) | ✅ |
| Frontend SDL com janela debug + log | ✅ |
| **Game Runtime (libcd32.a)** — kernel C freestanding | ✅ |
| **GPU Display List API** — CLEAR/RECT/TRI/LINE | ✅ |
| **Demos:** cubo 3D, triângulos, tech demo, software raster | ✅ |
| **ISO mastering** CD/DVD | ✅ |
| **ROM generator** (`--target hello|game`) | ✅ |
| **Docker PPC toolchain** (Ubuntu 24.04 + gcc-powerpc) | ✅ |
| **Bootstrap AROS** (boot chain alternativa, legado) | ✅ |

## Pipeline "liga o console → joga"

```bash
# 1. Build kernel + demo (via Docker, 5min setup)
make docker-build                # toolchain PPC
make docker-kernel               # kernel/demo.bin + ROM

# 2. Empacotar em ISO (CD ou DVD)
tools/mkcd.sh --media cd kernel/demo.elf rom/jogo.iso

# 3. Bootar no emulador
cargo run --release --features sdl-frontend -- \
  --bios rom/game_cd32.rom --disc rom/jogo.iso --sdl
```

O kernel (`kernel/kernel.c`) inicializa hardware, monta CD/DVD, carrega
`GAME.ELF` via ISO9660 + ELF loader, e pula para `game_main()`.

---

## Compilando

```bash
# Emulador (Rust)
cargo build --release
cargo build --release --features sdl-frontend   # com SDL2

# Kernel (requer cross-compiler PPC ou Docker)
make docker-kernel    # via Docker
# ou localmente: make -C kernel demo CC=powerpc-linux-gnu-gcc
```

## Executando

```bash
make ci                        # Pipeline completo de validação
make sdl-game                  # Boot game demo com gráfico
make sdl-hello                 # Boot "Hello CD³²" (teste hardware)
make trace-hello CYCLES=50000  # Trace com disassembler
```

### CLI

```
Usage: ml-gd2-rs [OPTIONS]

  -b, --bios <BIOS>          ROM de boot (512KB)
  -d, --disc <DISC>          Imagem ISO9660
      --disc-type <TYPE>     auto|cd|dvd (auto detecta por tamanho)
  -c, --cycles <CYCLES>      Ciclos a executar (0 = boot completo)
  -v, --verbose              Modo verbose
      --trace                Trace com disassembler
      --sdl                  Frontend SDL
      --save-state <PATH>    Salvar estado
      --load-state <PATH>    Carregar estado
```

## API do Jogo (`cd32.h`)

```c
/* ── GPU Display List ─────────────────────────────────────────── */
cd32_dl_t *cd32_gfx_begin(void);
void cd32_gfx_clear(cd32_dl_t *dl, uint32_t color);
void cd32_gfx_rect(cd32_dl_t *dl, int x, int y, int w, int h, uint32_t color);
void cd32_gfx_tri(cd32_dl_t *dl, int x0, int y0, int x1, int y1, int x2, int y2, uint32_t color);
void cd32_gfx_line(cd32_dl_t *dl, int x0, int y0, int x1, int y1, uint32_t color);
void cd32_gfx_submit(cd32_dl_t *dl);

/* ── Input digital + analógico ────────────────────────────────── */
void     cd32_input_init(void);
void     cd32_input_poll(void);
uint16_t cd32_joypad_read(void);
int16_t  cd32_analog_read(int axis);  // 0=LX, 1=LY, 2=RX, 3=RY

/* ── Áudio 8 canais (16-bit, 44.1kHz) ─────────────────────────── */
void cd32_audio_play(int ch, int16_t *samples, int count, int loop);
void cd32_audio_stop(int ch);
void cd32_audio_volume(int ch, int vol);  // 0-1024
void cd32_audio_pan(int ch, int pan);     // 0-255

/* ── CD / DVD ─────────────────────────────────────────────────── */
int  cd32_cdrom_init(void);
void *cd32_cdrom_load(const char *path);  // carrega GAME.ELF

/* ── Memory Card (2 slots × 512KB) ────────────────────────────── */
int cd32_memcard_init(void);
int cd32_memcard_present(int slot);
int cd32_memcard_read(int slot, uint32_t block, void *buf);
int cd32_memcard_write(int slot, uint32_t block, const void *buf);
```

## ABI (Application Binary Interface)

A struct `CD32Platform` em `0x0000_FC00` contém o mapa de hardware completo,
passada ao kernel via r3. ABI versionada e validada por teste.

```bash
make check-abi   # valida conformidade de offsets (17 campos, 68 bytes)
```

## Makefile

```bash
make build                    # Compila emulador
make headers                  # Gera headers ABI
make check-abi                # Valida conformidade
make rom-hello                # ROM "Hello CD³²"
make rom-game                 # ROM com game kernel
make test-hello               # Testa hello
make test-game                # Testa game demo
make sdl-game                 # Frontend gráfico com demo
make docker-build             # Builda toolchain PPC (Docker)
make docker-kernel            # Builda kernel + gera ROM via Docker
make ci                       # Pipeline completo
make clean                    # Limpa artefatos
```

## Estrutura do Projeto

```
ml-gd2/
├── Cargo.toml
├── Makefile
├── LICENSE
│
├── src/                          # Emulador (Rust)
│   ├── main.rs                   # CLI + frontend SDL
│   ├── bus.rs                    # MIU, mailbox, periféricos
│   ├── memory.rs                 # Mapa de memória (28MB unified)
│   ├── hardware.rs               # Boot orquestrado PPC/CF
│   ├── interrupt.rs              # 8 níveis de IRQ
│   ├── dma.rs                    # DMA 4 canais
│   ├── cdrom.rs                  # CD-ROM + ISO9660
│   ├── save.rs                   # Save states (9 seções)
│   ├── memcard.rs                # Memory Card (2 slots)
│   ├── disasm.rs                 # Disassembler PPC + ColdFire
│   ├── cpu/
│   │   ├── ppc603e.rs            # PPC603e + MMU/BAT
│   │   └── coldfire.rs           # ColdFire V4e
│   ├── gpu/tbdr.rs               # GPU Lisa II TBDR
│   └── audio/dsp.rs              # DSP áudio 8 canais
│
├── kernel/                       # Game runtime (libcd32.a)
│   ├── kernel.c                  # Entry point + boot
│   ├── video.c                   # Framebuffer + printf
│   ├── gfx.c                     # Display List GPU
│   ├── input.c                   # Joypad digital + analógico
│   ├── audio.c                   # DSP 8 canais (DMA)
│   ├── cdrom.c                   # CD/DVD + ISO9660 + ELF
│   ├── dma.c                     # DMA helper
│   ├── memcard.c                 # Memory Card driver
│   ├── string.c                  # memset/memcpy
│   ├── pad.c                     # Estado do joypad (edge detect)
│   ├── linker.ld
│   ├── Makefile
│   └── demo/                     # Jogos exemplo
│       ├── demo.c                # Retângulos + input
│       ├── gfx_demo.c            # Cubo 3D via GPU (display list)
│       ├── cube.c                # Cubo 3D via software
│       ├── poly.c                # Polígonos animados
│       └── tech_demo.c           # Tech demo completo
│
├── include/
│   ├── cd32.h                    # API pública do SDK
│   ├── cd32_gfx.h                # Display List API
│   ├── cd32_pad.h                # Estado do joypad
│   └── cd32_memcard.h            # Memory Card API
│
├── docker/                       # Toolchain PPC (cross-compiler)
│   ├── Dockerfile
│   └── entrypoint.sh
│
├── tools/
│   ├── mkcd.sh                   # Mastering ISO9660 (CD/DVD)
│   ├── gen_headers.rs            # Gerador ABI headers
│   └── check_abi_conformance.rs  # Validador de offsets
│
├── src/bin/gen_rom.rs            # Gerador de ROMs
├── docs/
│   ├── hardware/
│   │   ├── memory_map.md
│   │   └── boot_sequence.md
│   └── aros/abi.md
└── rom/                          # ROMs geradas
```

## Boot Chain

```
Power On
  │
  ▼
ColdFire V4e
  ├── Copia bootstrap PPC + kernel da ROM para RAM
  ├── Escreve struct CD32Platform em 0x0000_FC00
  ├── Handoff → STOP
  │
  ▼
PPC603e bootstrap
  ├── Spin no handoff
  ├── Stack pointer (r1 = 0x01BF_0000)
  ├── Platform struct em r3
  └── Jump para kernel (0x0000_2000)
  │
  ▼
kernel.c:_start()
  ├── cd32_video_init()        → framebuffer 640×480
  ├── cd32_audio_init()        → DSP 8 canais
  ├── cd32_memcard_init()      → Memory Cards
  ├── cd32_cdrom_init()        → monta ISO9660 (CD/DVD)
  ├── cd32_cdrom_load("GAME.ELF") → ELF parser + DMA
  └── game_main()              → o jogo
```

## Hardware Especulado

| Componente | Amiga CD³² (1993) | Amiga FlameSnow |
|---|---|---|
| CPU | Motorola 68EC020 @ 14MHz | PowerPC 603e @ 266MHz |
| Coprocessador | Akiko (CD controller) | ColdFire V4e @ 140MHz |
| GPU | — (Akiko + framebuffer) | Lisa II TBDR, 6M polys/s |
| RAM | 2MB Chip RAM | 28MB unificada |
| VRAM | — (compartilhada) | 8MB (dentro da unified) |
| Áudio | Paula 4 canais 8-bit | DSP 8 canais 16-bit 44.1kHz |
| Mídia | CD-ROM 2x | CD-ROM 12x / DVD opcional |
| Storage | — | Memory Card 512KB × 2 |
| Controle | Digital 2 botões | Digital + Analógico 4 eixos |
| SO | Kickstart/AmigaOS | Runtime próprio + AROS (legacy) |

## Licença

O código original do projeto (emulador, runtime, ferramentas) é MIT.
Componentes derivados de AROS seguem a AROS Public License (APL).
