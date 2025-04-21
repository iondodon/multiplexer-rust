import asyncio
import sys
import threading
from datetime import datetime
from colorama import Fore, Style, init

# Initialize colorama for colored output
init()

# Force unbuffered output
sys.stdout.reconfigure(line_buffering=True)

COLORS = [
    Fore.RED,
    Fore.GREEN,
    Fore.YELLOW,
    Fore.BLUE,
    Fore.MAGENTA,
    Fore.CYAN,
]

async def client(client_id):
    try:
        # Connect to the server
        reader, writer = await asyncio.open_connection('127.0.0.1', 8080)
        color = COLORS[client_id % len(COLORS)]
        thread_name = threading.current_thread().name
        print(f"{color}[Thread {thread_name}] Client {client_id} connected{Style.RESET_ALL}")

        # Read messages continuously
        message_count = 0
        while True:
            try:
                # Read a line (message ends with newline)
                data = await reader.readuntil(b'\n')
                if not data:
                    print(f"{color}[Thread {thread_name}] Server closed connection for Client {client_id}{Style.RESET_ALL}")
                    break
                
                message = data.decode().strip()
                message_count += 1
                timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
                print(f"{color}[Thread {thread_name} | Client {client_id}] {timestamp} -> {message!r} (msg #{message_count}){Style.RESET_ALL}", flush=True)
            except asyncio.IncompleteReadError:
                print(f"{color}[Thread {thread_name}] Server closed connection for Client {client_id}{Style.RESET_ALL}")
                break
            except asyncio.LimitOverrunError:
                print(f"{color}[Thread {thread_name}] Message too long for Client {client_id}{Style.RESET_ALL}")
                continue
            except Exception as e:
                print(f"{color}[Thread {thread_name}] Error for Client {client_id}: {e}{Style.RESET_ALL}")
                break

    except ConnectionRefusedError:
        print(f"{Fore.RED}[Thread {thread_name}] Failed to connect client {client_id}. Is the server running?{Style.RESET_ALL}")
    except Exception as e:
        print(f"{Fore.RED}[Thread {thread_name}] Client {client_id} error: {e}{Style.RESET_ALL}")
    finally:
        try:
            writer.close()
            await writer.wait_closed()
        except:
            pass
        print(f"{color}[Thread {thread_name}] Client {client_id} disconnected{Style.RESET_ALL}")

def run_client(client_id):
    asyncio.run(client(client_id))

async def main():
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <number_of_clients>")
        sys.exit(1)
    
    num_clients = int(sys.argv[1])
    threads = []
    for i in range(num_clients):
        thread = threading.Thread(target=run_client, args=(i,), name=f"ClientThread-{i}")
        thread.start()
        threads.append(thread)
    
    # Wait for all threads to complete
    for thread in threads:
        thread.join()

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nShutting down clients...") 