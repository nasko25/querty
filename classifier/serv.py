#import xmlrpc.client
from xmlrpc.server import SimpleXMLRPCServer
from xmlrpc.server import SimpleXMLRPCRequestHandler
import classify

# Restrict to a particular path.
class RequestHandler(SimpleXMLRPCRequestHandler):
    rpc_paths = ('/classifier')

# TODO the server should only run locally:
#       only the querty rust server should be able to make requests to the xmlrpc server

# Create server
with SimpleXMLRPCServer(('127.0.0.1', 9999),
                        requestHandler=RequestHandler) as server:
    server.register_introspection_functions()

    server.register_function(classify.classify)

    print("Server running")

    # Run the server's main loop
    server.serve_forever()

