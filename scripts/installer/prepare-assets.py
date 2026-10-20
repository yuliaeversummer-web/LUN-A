"""Lossless source copy is kept separately. Only resize/composite installer art.
No generative operations, recoloring, cropping or changes to the eight app poses.
Usage: python scripts/installer/prepare-assets.py
"""
from pathlib import Path
from PIL import Image
import hashlib, json

root = Path(__file__).resolve().parents[2]
assets = root / 'src-tauri/installer/assets'
source = assets / 'installer-original.png'
image = Image.open(source).convert('RGBA')
background = (13, 18, 22, 255)
canvas = Image.new('RGBA', (600, 668), background)
art = image.copy()
art.thumbnail((568, 584), Image.Resampling.LANCZOS)
canvas.alpha_composite(art, ((600-art.width)//2, (668-art.height)//2))
canvas.convert('RGB').save(assets / 'mascot-panel.bmp')
# Header has the same untouched character, with generous padding, not a crop.
header = Image.new('RGBA', (300, 114), background)
mini = image.copy()
mini.thumbnail((112, 104), Image.Resampling.LANCZOS)
header.alpha_composite(mini, (300-mini.width-8, (114-mini.height)//2))
header.convert('RGB').save(assets / 'header.bmp')
optimized = image.copy()
optimized.thumbnail((900, 900), Image.Resampling.LANCZOS)
optimized.save(assets / 'installer.png', optimize=True)
metadata = {'originalSha256': hashlib.sha256(source.read_bytes()).hexdigest(),
 'dimensions': image.size, 'sourceBytes': source.stat().st_size,
 'alphaExtrema': image.getchannel('A').getextrema(),
 'optimizedDimensions': optimized.size,
 'operations': 'proportional LANCZOS resize; bitmap alpha composition on #0d1216; no crop/recolor'}
(assets / 'metadata.json').write_text(json.dumps(metadata, indent=2), encoding='utf-8')
print(json.dumps(metadata))
