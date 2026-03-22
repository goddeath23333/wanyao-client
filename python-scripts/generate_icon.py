import os
import sys
from PIL import Image
from rembg import remove

SOURCE_IMAGE = r"H:\wanyao\wanyao-client\css\logtexture\image_962346168474464.png"
OUTPUT_DIR = r"H:\wanyao\wanyao-client\src-tauri\icons"

ICON_SIZES = {
    "32x32.png": (32, 32),
    "128x128.png": (128, 128),
    "128x128@2x.png": (256, 256),
    "icon.png": (512, 512),
    "Square30x30Logo.png": (30, 30),
    "Square44x44Logo.png": (44, 44),
    "Square71x71Logo.png": (71, 71),
    "Square89x89Logo.png": (89, 89),
    "Square107x107Logo.png": (107, 107),
    "Square142x142Logo.png": (142, 142),
    "Square150x150Logo.png": (150, 150),
    "Square284x284Logo.png": (284, 284),
    "Square310x310Logo.png": (310, 310),
    "StoreLogo.png": (50, 50),
}

def remove_background(input_path, output_path):
    with open(input_path, "rb") as f:
        input_data = f.read()
    output_data = remove(input_data)
    with open(output_path, "wb") as f:
        f.write(output_data)
    print(f"背景已移除: {output_path}")

def create_icon(source_path, output_path, size):
    img = Image.open(source_path)
    if img.mode == "RGBA":
        background = Image.new("RGBA", img.size, (0, 0, 0, 0))
        background.paste(img, mask=img.split()[3])
        img = background
    img_resized = img.resize(size, Image.Resampling.LANCZOS)
    img_resized.save(output_path, "PNG")
    print(f"已生成: {output_path} ({size[0]}x{size[1]})")

def create_ico(source_path, output_path):
    img = Image.open(source_path)
    sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    icons = []
    for size in sizes:
        resized = img.resize(size, Image.Resampling.LANCZOS)
        icons.append(resized)
    icons[0].save(
        output_path,
        format="ICO",
        sizes=sizes,
        append_images=icons[1:]
    )
    print(f"已生成 ICO: {output_path}")

def create_icns(source_path, output_path):
    img = Image.open(source_path)
    temp_dir = os.path.join(OUTPUT_DIR, "icon.iconset")
    os.makedirs(temp_dir, exist_ok=True)
    
    icns_sizes = [
        ("icon_16x16.png", 16),
        ("icon_16x16@2x.png", 32),
        ("icon_32x32.png", 32),
        ("icon_32x32@2x.png", 64),
        ("icon_128x128.png", 128),
        ("icon_128x128@2x.png", 256),
        ("icon_256x256.png", 256),
        ("icon_256x256@2x.png", 512),
        ("icon_512x512.png", 512),
        ("icon_512x512@2x.png", 1024),
    ]
    
    for filename, size in icns_sizes:
        resized = img.resize((size, size), Image.Resampling.LANCZOS)
        resized.save(os.path.join(temp_dir, filename), "PNG")
    
    print(f"已生成 iconset 目录: {temp_dir}")
    print("注意: .icns 文件需要在 macOS 上使用 iconutil 命令生成")
    print("在 macOS 上运行: iconutil -c icns icon.iconset")

def main():
    print("=" * 50)
    print("图标生成工具")
    print("=" * 50)
    
    temp_no_bg = os.path.join(OUTPUT_DIR, "temp_no_bg.png")
    
    print("\n步骤 1: 去除背景...")
    remove_background(SOURCE_IMAGE, temp_no_bg)
    
    print("\n步骤 2: 生成各尺寸 PNG 图标...")
    for filename, size in ICON_SIZES.items():
        output_path = os.path.join(OUTPUT_DIR, filename)
        create_icon(temp_no_bg, output_path, size)
    
    print("\n步骤 3: 生成 ICO 文件...")
    create_ico(temp_no_bg, os.path.join(OUTPUT_DIR, "icon.ico"))
    
    print("\n步骤 4: 生成 ICNS 资源...")
    create_icns(temp_no_bg, os.path.join(OUTPUT_DIR, "icon.icns"))
    
    if os.path.exists(temp_no_bg):
        os.remove(temp_no_bg)
        print(f"\n已清理临时文件: {temp_no_bg}")
    
    print("\n" + "=" * 50)
    print("所有图标已生成完成!")
    print("=" * 50)

if __name__ == "__main__":
    main()
