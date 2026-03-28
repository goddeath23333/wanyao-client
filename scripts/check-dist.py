import os
import filecmp
import hashlib

def get_file_hash(filepath):
    with open(filepath, 'rb') as f:
        return hashlib.md5(f.read()).hexdigest()

def compare_files(src, dst):
    if not os.path.exists(src):
        return 'SRC_MISSING'
    if not os.path.exists(dst):
        return 'DST_MISSING'
    
    src_hash = get_file_hash(src)
    dst_hash = get_file_hash(dst)
    
    if src_hash == dst_hash:
        return 'MATCH'
    else:
        return 'DIFFERENT'

def check_html_refs(html_path, base_dir):
    refs = []
    with open(html_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    import re
    js_refs = re.findall(r'<script src="([^"]+\.js)">', content)
    css_refs = re.findall(r'<link[^>]+href="([^"]+\.css)"', content)
    
    print('\n=== HTML 引用检查 ===')
    print(f'HTML文件: {html_path}')
    
    all_ok = True
    for ref in js_refs + css_refs:
        full_path = os.path.join(base_dir, ref)
        exists = os.path.exists(full_path)
        status = 'OK' if exists else 'MISSING'
        if not exists:
            all_ok = False
        print(f'  [{status}] {ref} -> {full_path}')
    
    return all_ok

def main():
    root = r'H:\wanyao\wanyao-client'
    dist_dir = os.path.join(root, 'dist')
    src_js_dir = os.path.join(root, 'js')
    src_css_dir = os.path.join(root, 'css')
    dist_js_dir = os.path.join(dist_dir, 'js')
    dist_css_dir = os.path.join(dist_dir, 'css')
    
    print('=' * 60)
    print('Wanyao Client 构建产物检查')
    print('=' * 60)
    
    print('\n=== JS 文件对比 ===')
    js_files = ['common.js', 'window-controls.js', 'system-monitor.js', 'python-module.js', 'serial.js']
    for js in js_files:
        src = os.path.join(src_js_dir, js)
        dst = os.path.join(dist_js_dir, js)
        status = compare_files(src, dst)
        status_text = {
            'MATCH': 'OK - 一致',
            'DIFFERENT': 'WARN - 不一致!',
            'SRC_MISSING': 'ERROR - 源文件缺失',
            'DST_MISSING': 'ERROR - 目标文件缺失'
        }.get(status, status)
        print(f'  [{status}] {js}: {status_text}')
    
    print('\n=== CSS 文件对比 ===')
    css_files = ['common.css', 'theme.css']
    for css in css_files:
        src = os.path.join(src_css_dir, css)
        dst = os.path.join(dist_css_dir, css)
        status = compare_files(src, dst)
        status_text = {
            'MATCH': 'OK - 一致',
            'DIFFERENT': 'WARN - 不一致!',
            'SRC_MISSING': 'ERROR - 源文件缺失',
            'DST_MISSING': 'ERROR - 目标文件缺失'
        }.get(status, status)
        print(f'  [{status}] {css}: {status_text}')
    
    print('\n=== HTML 文件对比 ===')
    src_html = os.path.join(root, 'index.html')
    dst_html = os.path.join(dist_dir, 'index.html')
    status = compare_files(src_html, dst_html)
    status_text = {
        'MATCH': 'OK - 一致',
        'DIFFERENT': 'WARN - 不一致!',
        'SRC_MISSING': 'ERROR - 源文件缺失',
        'DST_MISSING': 'ERROR - 目标文件缺失'
    }.get(status, status)
    print(f'  [{status}] index.html: {status_text}')
    
    check_html_refs(dst_html, dist_dir)
    
    print('\n=== dist 目录结构 ===')
    for root_dir, dirs, files in os.walk(dist_dir):
        level = root_dir.replace(dist_dir, '').count(os.sep)
        indent = '  ' * level
        print(f'{indent}{os.path.basename(root_dir)}/')
        subindent = '  ' * (level + 1)
        for file in files:
            filepath = os.path.join(root_dir, file)
            size = os.path.getsize(filepath)
            print(f'{subindent}{file} ({size} bytes)')
    
    print('\n=== 检查 window 函数导出 ===')
    for js in js_files:
        dst = os.path.join(dist_js_dir, js)
        if os.path.exists(dst):
            with open(dst, 'r', encoding='utf-8') as f:
                content = f.read()
            exports = [line.strip() for line in content.split('\n') if 'window.' in line and '=' in line and not line.strip().startswith('//')]
            print(f'\n  {js}:')
            for exp in exports[:5]:
                print(f'    {exp}')
            if len(exports) > 5:
                print(f'    ... 共 {len(exports)} 个导出')
    
    print('\n' + '=' * 60)
    print('检查完成')

if __name__ == '__main__':
    main()
