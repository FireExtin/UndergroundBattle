import json
import os
import urllib.request
import urllib.parse
import time
import concurrent.futures

# 读取cards.json文件
with open('resource/ymsj-fun.github.io/cards/cards.json', 'r', encoding='utf-8') as f:
    cards = json.load(f)

# 创建存放卡牌图片的目录
compressed_dir = 'resource/ymsj-fun.github.io/cards/compressed'
original_dir = 'resource/ymsj-fun.github.io/cards'

os.makedirs(compressed_dir, exist_ok=True)
os.makedirs(original_dir, exist_ok=True)

# 下载函数（带重试机制）
def download_with_retry(url, path, max_retries=3):
    for i in range(max_retries):
        try:
            urllib.request.urlretrieve(url, path)
            return True
        except Exception as e:
            if i < max_retries - 1:
                # 静默重试，只在最后一次失败时打印
                time.sleep(1)  # 暂停1秒后重试
            else:
                print(f"下载失败: {url} - {e}")
                return False

# 处理单张卡牌的下载
def process_card(card_id, card):
    name = card['name']
    
    # 构建图片URL（对文件名进行URL编码）
    filename = f"{card_id} {name}.jpg"
    encoded_filename = urllib.parse.quote(filename)
    compressed_url = f"https://ymsj-fun.github.io/cards/compressed/{encoded_filename}"
    original_url = f"https://ymsj-fun.github.io/cards/{encoded_filename}"
    
    # 构建本地文件路径
    compressed_path = os.path.join(compressed_dir, filename)
    original_path = os.path.join(original_dir, filename)
    
    # 检查文件是否已存在
    if os.path.exists(compressed_path) and os.path.exists(original_path):
        return False, True  # 失败标志, 跳过标志
    
    success = False
    
    # 下载压缩版图片
    if not os.path.exists(compressed_path):
        if download_with_retry(compressed_url, compressed_path):
            success = True
    else:
        success = True  # 压缩版已存在，视为成功
    
    # 下载原始版图片
    if not os.path.exists(original_path):
        download_with_retry(original_url, original_path)
    
    return success, False

# 下载卡牌图片
total = len(cards)
success_count = 0
skipped_count = 0

print(f"开始下载 {total} 张卡牌图片...")
print("使用多线程并行下载，这将提高下载速度...")

# 使用线程池并行下载
with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
    # 提交所有任务
    future_to_card = {executor.submit(process_card, card_id, card): (card_id, card['name']) 
                     for card_id, card in cards.items()}
    
    # 处理结果
    for i, future in enumerate(concurrent.futures.as_completed(future_to_card), 1):
        card_id, name = future_to_card[future]
        try:
            success, skipped = future.result()
            if skipped:
                skipped_count += 1
                print(f"[{i}/{total}] 跳过: {card_id} {name}（文件已存在）")
            else:
                if success:
                    success_count += 1
                print(f"[{i}/{total}] 处理: {card_id} {name}")
        except Exception as e:
            print(f"[{i}/{total}] 错误: {card_id} {name} - {e}")

print(f"\n下载完成！成功下载 {success_count} 张卡牌图片，跳过 {skipped_count} 张已存在的图片")
