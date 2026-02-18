import sys
from pyzk2 import ZK

def debug():
    ip = "192.168.12.13"
    if len(sys.argv) > 1: ip = sys.argv[1]
    
    print(f"Connecting to {ip}...")
    zk = ZK(ip, port=4370, timeout=10, force_udp=True, ommit_ping=True)
    
    try:
        conn = zk.connect()
        print("Connected!")
        
        print(f"MAC: {conn.get_mac() if hasattr(conn, 'get_mac') else 'N/A'}")
        print(f"SN: {conn.get_serialnumber() if hasattr(conn, 'get_serialnumber') else 'N/A'}")
        print(f"Name: {conn.get_device_name() if hasattr(conn, 'get_device_name') else 'N/A'}")
        print(f"Platform: {conn.get_platform() if hasattr(conn, 'get_platform') else 'N/A'}")
        print(f"FW: {conn.get_firmware_version() if hasattr(conn, 'get_firmware_version') else 'N/A'}")
        
        conn.disconnect()
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    debug()
