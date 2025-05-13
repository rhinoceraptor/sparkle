// Generate BMP files from PNGs with white backgrounds:
// $ for f in /path/to/*.png; do convert "$f" -depth 1 "${f%.png}.bmp"; done
// Then run script for each BMP:
// $ for f in /path/to/*.bmp; do node bmp2rust.js "$f" >> /path/to/file.rs; done

const assert = require("assert");
const fs = require("fs");
const path = require("path");
const { Jimp } = require("jimp");

async function main() {
    const args = process.argv.slice(2);
    if (args.length < 1) {
        console.error('Usage: bmp-to-rust <bmp-file> [--threshold N]');
        process.exit(1);
    }

    const filePath = args[0];
    // Optional threshold flag
    let threshold = 0;
    const thIndex = args.indexOf('--threshold');
    if (thIndex !== -1 && args.length > thIndex + 1) {
        threshold = parseInt(args[thIndex + 1], 10) || threshold;
    }

    try {
        const image = await Jimp.read(filePath);
        const { width, height, data } = image.bitmap;
        const bytes = [];

        for (let y = 0; y < height; y++) {
            for (let x0 = 0; x0 < width; x0 += 8) {
                let byte = 0;
                for (let bit = 0; bit < 8; bit++) {
                    const x = x0 + bit;
                    if (x < width) {
                        const idx = (y * width + x) * 4;
                        const r = image.bitmap.data[idx];
                        // Depth-1 BMP: red channel is 0 (black) or 255 (white)
                        assert(r === 0 || r === 255);
                        if (r === 0) {
                            byte |= 1 << (7 - bit);
                        }
                    }
                }
                bytes.push(byte);
            }
        }

        // Derive Rust identifier from filename
        const base = path.basename(filePath, path.extname(filePath));
        const ident = base.replace(/[^a-zA-Z0-9]/g, '_').toUpperCase();

        // Print Rust code
        process.stdout.write(`pub const ${ident}: [u8; ${bytes.length}] = [ `);
        let str_bytes = bytes.map(b => '0x' + b.toString(16).padStart(2, '0'));

        for (let i = 0; i < (bytes.length / 16); i += 16) {
            process.stdout.write(`${str_bytes.slice(i, i + 16).join(', ')}`);
        }

        console.log(' ];');
    } catch (err) {
        console.error('Error reading image:', err.message);
        process.exit(1);
    }
}

main();

