import socketserver
from time import sleep

class MyTCPHandler(socketserver.StreamRequestHandler):
    def handle(self):
        print("Got an TCP Message from {}".format(self.client_address[0]))
        sleep(1)
        msgRecvd = self.rfile.readline().strip()
        print("The Message is {}".format(msgRecvd))
        self.wfile.write("Hello TCP Client! I received a message from you!".encode())

PORT = 1235
with socketserver.TCPServer(("", PORT), MyTCPHandler) as httpd:
    print("serving at port", PORT)
    httpd.serve_forever()