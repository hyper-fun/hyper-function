from io import BytesIO
from model import Model
import msgpack
import hfn_core


class Context:
    def __init__(self, **kwargs):
        self.package_id = kwargs.get('package_id')
        self.socket_id = kwargs.get('socket_id')
        self.headers = kwargs.get('headers')
        self.cookies = kwargs.get('cookies')
        self.data = kwargs.get('data')
        self.module_id = kwargs.get('module_id')
        self.hfn_id = kwargs.get('hfn_id')
        self.schemas = kwargs.get('schemas')

    def set_state(self, state):
        module_id = state.schema.get('module_id')

        if not module_id:
            return

        payload = BytesIO()
        payload.write(msgpack.packb(2))
        payload.write(msgpack.packb(state.schema['pkg_id']))
        payload.write(msgpack.packb(module_id))
        payload.write(msgpack.packb(state.encode()))

        buf = BytesIO()
        buf.write(msgpack.packb(0))
        buf.write(msgpack.packb(self.package_id))
        buf.write(msgpack.packb({}))
        buf.write(msgpack.packb(payload.getvalue()))

        hfn_core.send_message(self.socket_id, buf.getvalue())

    def model(self, name):
        schema = self.schemas.get(name)
        if schema == None:
            return None

        model = Model(schema, self.schemas)

        return model
