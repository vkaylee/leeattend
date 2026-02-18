import sys
import logging
from pyzk2 import ZK

# Setup logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(message)s')

def check_device(ip, port=4370, introspect=False):
    print(f"📡 Đang kết nối tới {ip}:{port}...")
    zk = ZK(ip, port=port, timeout=10, force_udp=True, ommit_ping=True)
    
    try:
        conn = zk.connect()
        print("✅ KẾT NỐI THÀNH CÔNG!")
        
        if introspect:
            print("\n--- [🔍] Phân tích các hàm có sẵn trên đối tượng 'conn' ---")
            
            # Lọc ra các hàm có thể gọi (callable) và không phải là internal (__method__)
            methods = [
                m for m in dir(conn) 
                if callable(getattr(conn, m)) and not m.startswith('__')
            ]
            
            # Phân loại và hiển thị
            get_methods = sorted([m for m in methods if m.startswith('get_')])
            set_methods = sorted([m for m in methods if m.startswith('set_')])
            other_methods = sorted([m for m in methods if not m.startswith('get_') and not m.startswith('set_')])

            print("\n  --- Getters (Lấy thông tin) ---")
            for m in get_methods: print(f"    - {m}")

            print("\n  --- Setters (Thiết lập) ---")
            for m in set_methods: print(f"    - {m}")
                
            print("\n  --- Actions (Hành động khác) ---")
            for m in other_methods: print(f"    - {m}")
            
            print("\n----------------------------------------------------")
            
        else:
            # (Phần check cũ để giữ lại)
            pass

        conn.disconnect()

    except Exception as e:
        print(f"\n❌ KHÔNG THỂ KẾT NỐI: {e}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 check_device.py <IP> [PORT] [--introspect]")
        sys.exit(1)
        
    target_ip = sys.argv[1]
    target_port = 4370
    do_introspect = False

    if len(sys.argv) > 2:
        if sys.argv[2] == '--introspect':
            do_introspect = True
        else:
            try:
                target_port = int(sys.argv[2])
                if len(sys.argv) > 3 and sys.argv[3] == '--introspect':
                    do_introspect = True
            except ValueError:
                print(f"Error: Invalid port '{sys.argv[2]}'")
                sys.exit(1)
                
    check_device(target_ip, target_port, introspect=do_introspect)
