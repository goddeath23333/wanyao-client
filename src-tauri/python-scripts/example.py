def calculate(a, b):
    return str(int(a) + int(b))

if __name__ == "__main__":
    import sys
    if len(sys.argv) >= 3:
        result = calculate(sys.argv[1], sys.argv[2])
        print(f"计算结果: {result}")
    else:
        print("用法: python example.py <数字1> <数字2>")
