def check_type(value, type):
    if type == 's':
        return isinstance(value, str)
    elif type == 'i':
        return isinstance(value, int) and value <= 2147483647 and value >= -2147483648
    elif type == 'f':
        return isinstance(value, float)
    elif type == 'b':
        return isinstance(value, bool)
    elif type == 't':
        return isinstance(value, bytes)
    else:
        return False


class Model:
    data = {}
    schema = {}
    schemas = {}

    def __init__(self, schema, schemas):
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
                    if not check_type(v, field.type):
                        return False
            else:
                if not check_type(value, field.type):
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
        self.data.setdefault(key, value)
        return True
