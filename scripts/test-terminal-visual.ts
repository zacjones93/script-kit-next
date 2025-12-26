// Name: Terminal Visual Test
// Description: Display various terminal features for manual verification

import '../scripts/kit-sdk';

/**
 * Terminal Visual Test Script
 * 
 * This script demonstrates various terminal features implemented in Tier 1:
 * - ANSI color codes (foreground and background)
 * - Text attributes (bold, underline, italic)
 * - 256-color palette
 * - True color (24-bit RGB)
 * - Cursor positioning
 * 
 * Run with: ./target/debug/script-kit-gpui scripts/test-terminal-visual.ts
 */

// Test 1: Basic ANSI Colors (8 colors)
const basicColorsCommand = `
echo "=== BASIC ANSI COLORS (8 colors) ==="
echo ""
printf '\\x1b[30mBlack\\x1b[0m   '
printf '\\x1b[31mRed\\x1b[0m     '
printf '\\x1b[32mGreen\\x1b[0m   '
printf '\\x1b[33mYellow\\x1b[0m  '
printf '\\x1b[34mBlue\\x1b[0m    '
printf '\\x1b[35mMagenta\\x1b[0m '
printf '\\x1b[36mCyan\\x1b[0m    '
printf '\\x1b[37mWhite\\x1b[0m'
echo ""
echo ""
`;

// Test 2: Bright/Bold Colors
const brightColorsCommand = `
echo "=== BRIGHT COLORS ==="
echo ""
printf '\\x1b[90mBright Black\\x1b[0m   '
printf '\\x1b[91mBright Red\\x1b[0m     '
printf '\\x1b[92mBright Green\\x1b[0m   '
printf '\\x1b[93mBright Yellow\\x1b[0m  '
echo ""
printf '\\x1b[94mBright Blue\\x1b[0m    '
printf '\\x1b[95mBright Magenta\\x1b[0m '
printf '\\x1b[96mBright Cyan\\x1b[0m    '
printf '\\x1b[97mBright White\\x1b[0m'
echo ""
echo ""
`;

// Test 3: Background Colors
const bgColorsCommand = `
echo "=== BACKGROUND COLORS ==="
echo ""
printf '\\x1b[40m Black BG \\x1b[0m '
printf '\\x1b[41m Red BG \\x1b[0m '
printf '\\x1b[42m Green BG \\x1b[0m '
printf '\\x1b[43m Yellow BG \\x1b[0m '
echo ""
printf '\\x1b[44m Blue BG \\x1b[0m '
printf '\\x1b[45m Magenta BG \\x1b[0m '
printf '\\x1b[46m Cyan BG \\x1b[0m '
printf '\\x1b[47m White BG \\x1b[0m'
echo ""
echo ""
`;

// Test 4: Text Attributes
const textAttributesCommand = `
echo "=== TEXT ATTRIBUTES ==="
echo ""
printf '\\x1b[1mBold text\\x1b[0m            '
printf '\\x1b[2mDim text\\x1b[0m             '
printf '\\x1b[3mItalic text\\x1b[0m          '
echo ""
printf '\\x1b[4mUnderline text\\x1b[0m       '
printf '\\x1b[5mBlink text\\x1b[0m           '
printf '\\x1b[7mInverse text\\x1b[0m         '
echo ""
printf '\\x1b[9mStrikethrough\\x1b[0m        '
printf '\\x1b[1;4mBold+Underline\\x1b[0m     '
printf '\\x1b[1;3;4mBold+Italic+UL\\x1b[0m'
echo ""
echo ""
`;

// Test 5: 256 Color Palette (sample)
const color256Command = `
echo "=== 256 COLOR PALETTE (sample) ==="
echo ""
echo "Standard colors (0-15):"
for i in {0..15}; do printf "\\x1b[48;5;%sm %3s \\x1b[0m" "$i" "$i"; done
echo ""
echo ""
echo "216 color cube sample (16-231):"
for i in {16..51}; do printf "\\x1b[48;5;%sm  \\x1b[0m" "$i"; done
echo ""
for i in {52..87}; do printf "\\x1b[48;5;%sm  \\x1b[0m" "$i"; done
echo ""
echo ""
echo "Grayscale (232-255):"
for i in {232..255}; do printf "\\x1b[48;5;%sm  \\x1b[0m" "$i"; done
echo ""
echo ""
`;

// Test 6: True Color (24-bit RGB)
const trueColorCommand = `
echo "=== TRUE COLOR (24-bit RGB) ==="
echo ""
echo "Red gradient:"
for i in $(seq 0 15 255); do printf "\\x1b[48;2;%s;0;0m \\x1b[0m" "$i"; done
echo ""
echo "Green gradient:"
for i in $(seq 0 15 255); do printf "\\x1b[48;2;0;%s;0m \\x1b[0m" "$i"; done
echo ""
echo "Blue gradient:"
for i in $(seq 0 15 255); do printf "\\x1b[48;2;0;0;%sm \\x1b[0m" "$i"; done
echo ""
echo "Rainbow:"
for i in $(seq 0 10 359); do
  r=$((128 + 127 * $(echo "c($i * 3.14159 / 180)" | bc -l | cut -c1-5 | tr -d '.')))
  g=$((128 + 127 * $(echo "c(($i + 120) * 3.14159 / 180)" | bc -l | cut -c1-5 | tr -d '.')))
  b=$((128 + 127 * $(echo "c(($i + 240) * 3.14159 / 180)" | bc -l | cut -c1-5 | tr -d '.')))
  printf "\\x1b[48;2;%s;%s;%sm \\x1b[0m" "$r" "$g" "$b"
done 2>/dev/null || echo "(Rainbow requires bc)"
echo ""
echo ""
`;

// Test 7: Combined foreground + background
const combinedColorsCommand = `
echo "=== COMBINED FG + BG COLORS ==="
echo ""
printf '\\x1b[31;47m Red on White \\x1b[0m  '
printf '\\x1b[37;44m White on Blue \\x1b[0m  '
printf '\\x1b[30;43m Black on Yellow \\x1b[0m  '
echo ""
printf '\\x1b[1;33;41m Bold Yellow on Red \\x1b[0m  '
printf '\\x1b[4;36;45m Underline Cyan on Magenta \\x1b[0m'
echo ""
echo ""
`;

// Test 8: Special Characters and Unicode
const unicodeCommand = `
echo "=== SPECIAL CHARACTERS ==="
echo ""
echo "Box drawing: ┌─────────┐"
echo "            │ Box     │"
echo "            └─────────┘"
echo ""
echo "Arrows: ← ↑ → ↓ ↔ ↕"
echo "Symbols: ★ ● ■ ♠ ♥ ♦ ♣"
echo "Math: ± × ÷ ≤ ≥ ≠ ∞ √"
echo ""
`;

// Combine all commands into one shell script
const fullTestCommand = `
clear
echo "╔════════════════════════════════════════════════════════════╗"
echo "║           TERMINAL VISUAL FEATURE TEST                     ║"
echo "║           Script Kit GPUI - Tier 1 Terminal                ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
${basicColorsCommand}
${brightColorsCommand}
${bgColorsCommand}
${textAttributesCommand}
${color256Command}
${combinedColorsCommand}
${unicodeCommand}
echo "=== TEST COMPLETE ==="
echo ""
echo "Press Enter to close terminal..."
`;

// Run the visual test
await term(fullTestCommand);
