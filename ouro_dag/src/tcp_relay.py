# tcp_relay.py -- simple TCP forwarder to relay connections between two endpoints
# Usage: python tcp_relay.py --listen 9000 --target host:port
import asyncio
import argparse

async def handle_client(reader, writer, target_host, target_port):
    try:
        r2, w2 = await asyncio.open_connection(target_host, target_port)
    except Exception as e:
        print("failed to connect to target:", e)
        writer.close()
        await writer.wait_closed()
        return

    async def pipe(src, dst):
        try:
            while True:
                data = await src.read(4096)
                if not data:
                    break
                dst.write(data)
                await dst.drain()
        except Exception as e:
            pass
        finally:
            try: dst.close()
            except: pass

    await asyncio.gather(pipe(reader, w2), pipe(r2, writer))
    try:
        writer.close()
        await writer.wait_closed()
    except:
        pass

async def main(listen_port, target):
    host, port = target.split(':')
    port = int(port)
    server = await asyncio.start_server(lambda r,w: handle_client(r, w, host, port), '0.0.0.0', listen_port)
    print(f"relay listening 0.0.0.0:{listen_port} -> {host}:{port}")
    async with server:
        await server.serve_forever()

if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('--listen', type=int, required=True)
    p.add_argument('--target', required=True)
    args = p.parse_args()
    asyncio.run(main(args.listen, args.target))
