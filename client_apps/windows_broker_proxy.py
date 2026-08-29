import asyncio
import logging
import sys

# Configure basic logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - [BROKER] - %(message)s')

# Windows Host IP and Port to listen on (Accessible from Smartphone via Wi-Fi)
LISTEN_HOST = '0.0.0.0'
LISTEN_PORT = 8081

# WSL Localhost IP and Port to forward to
WSL_HOST = '172.31.1.14'  # <-- Explicit WSL Linux VM IP
WSL_PORT = 8080

async def handle_client(reader, writer):
    client_addr = writer.get_extra_info('peername')
    logging.info(f"New connection from Smartphone App: {client_addr}")
    
    try:
        # Connect to WSL HETE Server
        wsl_reader, wsl_writer = await asyncio.open_connection(WSL_HOST, WSL_PORT)
        logging.info(f"Successfully connected to WSL HETE Server at {WSL_HOST}:{WSL_PORT}")
        
        async def forward(src_reader, dst_writer, direction):
            try:
                while True:
                    data = await src_reader.read(4096)
                    if not data:
                        break
                    dst_writer.write(data)
                    await dst_writer.drain()
                    # logging.info(f"Forwarded {len(data)} bytes {direction}")
            except Exception as e:
                logging.error(f"Error forwarding {direction}: {e}")
            finally:
                dst_writer.close()

        # Run bi-directional forwarding concurrently
        await asyncio.gather(
            forward(reader, wsl_writer, "Smartphone -> WSL"),
            forward(wsl_reader, writer, "WSL -> Smartphone")
        )
    except ConnectionRefusedError:
        logging.error(f"Cannot connect to WSL HETE Server at {WSL_HOST}:{WSL_PORT}. Is it running?")
    except Exception as e:
        logging.error(f"Unexpected error: {e}")
    finally:
        writer.close()
        logging.info(f"Connection closed for {client_addr}")

async def main():
    logging.info(f"Starting Windows Broker App...")
    server = await asyncio.start_server(handle_client, LISTEN_HOST, LISTEN_PORT)
    
    addrs = ', '.join(str(sock.getsockname()) for sock in server.sockets)
    logging.info(f"Broker listening on {addrs}")
    logging.info(f"Please set your Smartphone App IP to your Windows Wi-Fi IP and port {LISTEN_PORT}")
    
    async with server:
        await server.serve_forever()

if __name__ == '__main__':
    try:
        if sys.platform == 'win32':
            asyncio.set_event_loop_policy(asyncio.WindowsSelectorEventLoopPolicy())
        asyncio.run(main())
    except KeyboardInterrupt:
        logging.info("Broker stopped by user.")
