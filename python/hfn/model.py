import msgpack
from io import BytesIO


def check_type(value, field_type):
    if field_type == 's':
        return isinstance(value, str)
    elif field_type == 'i':
        return isinstance(value, int) and value <= 2147483647 and value >= -2147483648
    elif field_type == 'f':
        return isinstance(value, float)
    elif field_type == 'b':
        return isinstance(value, bool)
    elif field_type == 't':
        return isinstance(value, bytes)
    else:
        return False


class Model:
    def __init__(self, schema, schemas):
        self.data = {}
        self.schema = schema
        self.schemas = schemas

    def set(self, key, value):
        if not key or not value:
            return False

        field = self.schema['fields'].get(key)
        if not field:
            return False

        value_is_array = isinstance(value, list)
        if field['is_array'] != value_is_array:
            return

        if len(field['type']) == 1:
            if value_is_array:
                for v in value:
                    if not check_type(v, field['type']):
                        return False
            else:
                if not check_type(value, field['type']):
                    return False

        else:
            target_schema = self.schemas.get(field['type'])
            if not target_schema:
                return False
            if value_is_array:
                for v in value:
                    if not isinstance(v, Model):
                        return False
                    if v.schema['id'] != target_schema['id'] or v.schema['pkg_id'] != target_schema['pkg_id']:
                        return False
            else:
                if not isinstance(value, Model):
                    return False
                if value.schema['id'] != target_schema['id'] or value.schema['pkg_id'] != target_schema['pkg_id']:
                    return False

        self.data[key] = value
        return True

    def get(self, key):
        return self.data.get(key)

    def has(self, key):
        return self.data.get(key) != None

    def keys(self):
        return list(self.data.keys())

    def delete(self, key):
        if self.has(key):
            del self.data[key]

    def encode(self):
        buf = BytesIO()
        keys = self.keys()
        for key in keys:
            field = self.schema['fields'].get(key)

            buf.write(msgpack.packb(field['id']))

            value = self.data[key]
            if len(field['type']) == 1:
                buf.write(msgpack.packb(value))
            else:
                if field['is_array']:
                    arr = []
                    for v in value:
                        arr.append(v.encode())
                    buf.write(msgpack.packb(arr))
                else:
                    buf.write(msgpack.packb(value.encode()))
        return buf.getvalue()

    def decode(self, data):
        items = None

        try:
            unpacker = msgpack.Unpacker()
            unpacker.feed(data)
            items = list(unpacker)
        except:
            return False

        field = None
        for item in items:
            if field == None:
                field = self.schema['fields'].get(item)

                if not field:
                    return False
                continue

            value = None
            if len(field['type']) == 1:
                value = item
            else:
                target_schema = self.schemas.get(field['type'])
                if not target_schema:
                    return False

                if field['is_array']:
                    value = []
                    for v in item:
                        m = Model(target_schema, self.schemas)
                        m.decode(v)
                        value.append(m)
                else:
                    value = Model(target_schema, self.schemas)
                    value.decode(item)

            self.set(field['name'], value)
            field = None

    def to_dict(self):
        obj = {}
        for key in self.keys():
            field = self.schema['fields'].get(key)
            if not field:
                return False

            value = self.data[key]
            if len(field['type']) == 1:
                obj[field['name']] = value
            else:
                if field['is_array']:
                    arr = []
                    for v in value:
                        arr.append(v.to_dict())
                    obj[field['name']] = arr
                else:
                    obj[field['name']] = value.to_dict()
        return obj
