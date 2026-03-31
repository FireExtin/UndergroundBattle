import os
import json
import pdfplumber
import re

# 定义输出目录
output_dir = 'organized_content'
os.makedirs(output_dir, exist_ok=True)
os.makedirs(os.path.join(output_dir, 'rules'), exist_ok=True)
os.makedirs(os.path.join(output_dir, 'cards'), exist_ok=True)
os.makedirs(os.path.join(output_dir, 'tokens'), exist_ok=True)

# 解析PDF文件
def parse_pdf(pdf_path):
    text = ''
    with pdfplumber.open(pdf_path) as pdf:
        for page in pdf.pages:
            page_text = page.extract_text()
            if page_text:
                text += page_text + '\n\n'
    return text

# 清理文本，使其更适合AI阅读
def clean_text(text):
    # 移除多余的空白行
    text = re.sub(r'\n\s*\n', '\n\n', text)
    # 移除行首行尾的空白
    lines = text.split('\n')
    cleaned_lines = [line.strip() for line in lines]
    # 重新组合文本，保持适当的段落
    cleaned_text = '\n'.join(cleaned_lines)
    # 移除多余的空行
    cleaned_text = re.sub(r'\n{3,}', '\n\n', cleaned_text)
    return cleaned_text

# 解析并保存规则书
def process_rulebooks():
    rulebooks_dir = 'resource/ymsj-fun.github.io/public/docs'
    rulebooks = [
        '霸权说明书.pdf',
        '隐秘世界玩家指南.pdf',
        '隐秘世界规则手册.pdf',
        '隐秘世界勘误及释疑.pdf'
    ]
    
    for book in rulebooks:
        pdf_path = os.path.join(rulebooks_dir, book)
        if os.path.exists(pdf_path):
            print(f"解析规则书: {book}")
            text = parse_pdf(pdf_path)
            cleaned_text = clean_text(text)
            
            # 保存为Markdown文件
            md_filename = os.path.splitext(book)[0] + '.md'
            md_path = os.path.join(output_dir, 'rules', md_filename)
            with open(md_path, 'w', encoding='utf-8') as f:
                f.write(f"# {os.path.splitext(book)[0]}\n\n")
                f.write(cleaned_text)
            print(f"保存为: {md_path}")

# 处理卡片数据
def process_cards():
    cards_json_path = 'resource/ymsj-fun.github.io/cards/cards.json'
    if os.path.exists(cards_json_path):
        print("处理卡片数据...")
        with open(cards_json_path, 'r', encoding='utf-8') as f:
            cards = json.load(f)
        
        # 按类型组织卡片
        cards_by_type = {}
        for card_id, card in cards.items():
            card_type = card.get('basic-type', ['未知'])[0]
            if card_type not in cards_by_type:
                cards_by_type[card_type] = {}
            cards_by_type[card_type][card_id] = card
        
        # 保存组织后的卡片数据
        for card_type, type_cards in cards_by_type.items():
            type_dir = os.path.join(output_dir, 'cards', card_type)
            os.makedirs(type_dir, exist_ok=True)
            
            # 保存为JSON文件
            json_path = os.path.join(type_dir, 'cards.json')
            with open(json_path, 'w', encoding='utf-8') as f:
                json.dump(type_cards, f, ensure_ascii=False, indent=2)
            print(f"保存{card_type}卡片: {len(type_cards)}张")
            
            # 保存为Markdown文件，便于AI阅读
            md_path = os.path.join(type_dir, 'cards.md')
            with open(md_path, 'w', encoding='utf-8') as f:
                f.write(f"# {card_type}卡片\n\n")
                for card_id, card in type_cards.items():
                    f.write(f"## {card.get('name', '未知')} ({card_id})\n")
                    f.write(f"- 类型: {card.get('type', '未知')}\n")
                    f.write(f"- 费用: {card.get('cost', '未知')}\n")
                    if 'color' in card:
                        f.write(f"- 颜色: {card['color']}\n")
                    if 'magic' in card:
                        f.write(f"- 魔法领域: {card['magic']}\n")
                    if 'dfc' in card:
                        f.write(f"- 防御: {card['dfc']}\n")
                    if 'abl' in card:
                        f.write(f"- 能力: {card['abl']}\n")
                    if 'text' in card:
                        text = card['text'].replace('\n', ' ')
                        f.write(f"- 效果: {text}\n")
                    f.write("\n")

# 处理标志物数据
def process_tokens():
    cards_json_path = 'resource/ymsj-fun.github.io/cards/cards.json'
    if os.path.exists(cards_json_path):
        print("处理标志物数据...")
        with open(cards_json_path, 'r', encoding='utf-8') as f:
            cards = json.load(f)
        
        # 提取标志物
        tokens = {}
        for card_id, card in cards.items():
            if card.get('istoken', False):
                tokens[card_id] = card
        
        # 保存标志物数据
        if tokens:
            # 保存为JSON文件
            json_path = os.path.join(output_dir, 'tokens', 'tokens.json')
            with open(json_path, 'w', encoding='utf-8') as f:
                json.dump(tokens, f, ensure_ascii=False, indent=2)
            print(f"保存标志物: {len(tokens)}个")
            
            # 保存为Markdown文件，便于AI阅读
            md_path = os.path.join(output_dir, 'tokens', 'tokens.md')
            with open(md_path, 'w', encoding='utf-8') as f:
                f.write("# 标志物\n\n")
                for token_id, token in tokens.items():
                    f.write(f"## {token.get('name', '未知')} ({token_id})\n")
                    f.write(f"- 类型: {token.get('type', '未知')}\n")
                    if 'text' in token:
                        text = token['text'].replace('\n', ' ')
                        f.write(f"- 效果: {text}\n")
                    f.write("\n")

# 主函数
def main():
    print("开始组织内容...")
    process_rulebooks()
    process_cards()
    process_tokens()
    print("内容组织完成！")

if __name__ == "__main__":
    main()
