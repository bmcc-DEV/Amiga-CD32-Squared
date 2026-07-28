#!/bin/sh
# mkcd.sh — MontêLauro CD+G² ISO Mastering Tool
#
# Empacota um jogo (.elf) em imagem ISO9660.
# A desenvolvedora escolhe a midia:
#   --media cd   (default, ~700MB max)
#   --media dvd  (ate 4.7GB)
#
# Uso: ./tools/mkcd.sh [--media cd|dvd] kernel/demo/demo.elf [rom/jogo.iso]

set -e

MEDIA="cd"
ELF=""
OUT=""

while [ $# -gt 0 ]; do
    case "$1" in
        --media)
            MEDIA="$2"
            shift 2
            ;;
        *)
            if [ -z "$ELF" ]; then
                ELF="$1"
            elif [ -z "$OUT" ]; then
                OUT="$1"
            fi
            shift
            ;;
    esac
done

ELF="${ELF:-kernel/demo.elf}"
OUT="${OUT:-rom/jogo.iso}"
TMP=$(mktemp -d)

echo "=== MontêLauro CD+G² — ISO Mastering ==="
echo "Midia:  $MEDIA"
echo "Jogo:   $ELF"
echo "ISO:    $OUT"

if [ ! -f "$ELF" ]; then
    echo "ERRO: $ELF nao encontrado. Compile o jogo primeiro:"
    echo "  make -C kernel demo"
    exit 1
fi

# Monta diretorio com GAME.ELF na raiz
mkdir -p "$TMP/cd"
cp "$ELF" "$TMP/cd/GAME.ELF"

MKISOFS_OPTS="-o $OUT -V MONTELAURO -J -R -sysid CD32"
MKISOFS_OPTS="$MKISOFS_OPTS -volset 'MONTELAURO CD32' -publisher 'Montelauro CD+G2 Labs' -quiet"

if [ "$MEDIA" = "dvd" ]; then
    # DVD: permite arquivos maiores
    echo "Modo DVD: sem limite de tamanho (ISO9660 level 3)"
    MKISOFS_OPTS="$MKISOFS_OPTS -iso-level 3"
fi

eval mkisofs $MKISOFS_OPTS "$TMP/cd" 2>&1

rm -rf "$TMP"
ls -lh "$OUT"
echo "ISO pronta: $OUT"
echo "Teste CD:  cargo run --release --bios rom/game_cd32.rom --disc $OUT --sdl"
echo "Teste DVD: cargo run --release --bios rom/game_cd32.rom --disc $OUT --disc-type dvd --sdl"
