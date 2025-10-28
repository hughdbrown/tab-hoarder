#!/usr/bin/env python3
"""Generate placeholder icons for Tab Hoarder Chrome extension"""

import os

# Create simple SVG icons
def create_svg_icon(size):
    """Create a simple SVG icon with 'TH' text"""
    svg = f'''<?xml version="1.0" encoding="UTF-8"?>
<svg width="{size}" height="{size}" xmlns="http://www.w3.org/2000/svg">
  <rect width="{size}" height="{size}" fill="#5B4FE8" rx="8"/>
  <text x="50%" y="50%" font-family="Arial, sans-serif" font-size="{int(size * 0.5)}"
        font-weight="bold" fill="white" text-anchor="middle" dominant-baseline="central">
    TH
  </text>
</svg>'''
    return svg

def svg_to_png(svg_content, output_path, size):
    """Convert SVG to PNG using various methods"""
    try:
        # Try cairosvg first
        import cairosvg
        cairosvg.svg2png(bytestring=svg_content.encode('utf-8'),
                        write_to=output_path,
                        output_width=size,
                        output_height=size)
        return True
    except ImportError:
        pass

    try:
        # Try PIL with svglib
        from svglib.svglib import svg2rlg
        from reportlab.graphics import renderPM
        import tempfile

        # Write SVG to temp file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.svg', delete=False) as f:
            f.write(svg_content)
            temp_svg = f.name

        drawing = svg2rlg(temp_svg)
        renderPM.drawToFile(drawing, output_path, fmt="PNG")
        os.unlink(temp_svg)
        return True
    except ImportError:
        pass

    # If no conversion libraries available, just save SVG files
    return False

# Main
os.makedirs('icons', exist_ok=True)

sizes = [16, 48, 128]
can_convert = False

for size in sizes:
    svg_content = create_svg_icon(size)
    png_path = f'icons/icon{size}.png'
    svg_path = f'icons/icon{size}.svg'

    # Always save SVG
    with open(svg_path, 'w') as f:
        f.write(svg_content)
    print(f"Created {svg_path}")

    # Try to convert to PNG
    if svg_to_png(svg_content, png_path, size):
        print(f"Created {png_path}")
        can_convert = True
    else:
        # Copy SVG as PNG fallback (Chrome can handle SVG)
        import shutil
        shutil.copy(svg_path, png_path)
        print(f"Copied SVG to {png_path} (install cairosvg or svglib+reportlab for PNG conversion)")

print("\n✅ Icon generation complete!")
if not can_convert:
    print("💡 Tip: Install cairosvg for proper PNG icons:")
    print("   pip install cairosvg")
