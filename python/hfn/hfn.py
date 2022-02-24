import msgpack
import hfn_core
from model import Model
from context import Context
from concurrent.futures import ThreadPoolExecutor


def run(packages, **kwargs):
    main_pkg = list(filter(lambda x: x.name == '', packages))
    if len(main_pkg) == 0:
        raise Exception('No main package')

    main_pkg = main_pkg[0]
    pkg_names = list(map(lambda x: x.name, packages))

    result = msgpack.unpackb(hfn_core.init(msgpack.packb({
        "dev": kwargs.get('dev', True),
        "addr": "[::1]:3000",
        "sdk": "python-0.1.0",
        "hfn_config_path": "/Users/afei/Desktop/aefe/hfn.json",
        "pkg_names": pkg_names
    })))

    handlers = {}

    for pkg_config in result['packages']:
        pkg = list(filter(lambda x: x.name == pkg_config['name'], packages))
        if len(pkg) == 0:
            continue

        pkg = pkg[0]

        mod_configs = list(filter(lambda x: x['package_id'] ==
                           pkg_config['id'], result['modules']))

        for mod_config in mod_configs:
            mod = pkg.modules.get(mod_config['name'].lower())
            if mod == None:
                continue

            hfn_configs = list(filter(lambda x: x['module_id'] == mod_config['id']
                                      and x['package_id'] == pkg_config['id'], result['hfns']))

            for hfn_config in hfn_configs:
                hfn_exists = hfn_config['name'] in mod['methods']
                if not hfn_exists:
                    continue

                id = '-'.join([str(pkg_config['id']), str(mod_config['id']),
                               str(hfn_config['id'])])

                handlers[id] = getattr(
                    mod['instance'], hfn_config['name'])

    schemas = {}
    for schema_config in result['schemas']:
        schema = {
            'id': schema_config['id'],
            'pkg_id': schema_config['package_id'],
            'fields': {}
        }

        fields = list(
            filter(lambda x: x['schema_id'] == schema['id'] and x['package_id'] == schema['pkg_id'], result['fields']))

        for field_config in fields:
            field = {
                'id': field_config['id'],
                'name': field_config['name'],
                'type': field_config['t'],
                'is_array': field_config['is_array'],
                'pkg_id': field_config['package_id'],
                'schema_id': field_config['schema_id']
            }

            schema['fields'][field['id']] = field
            schema['fields'][field['name']] = field

        model = list(filter(
            lambda x: x['schema_id'] == schema['id'] and x['package_id'] == schema['pkg_id'], result['models']))

        if len(model) > 0:
            model = model[0]
            if model['name'] == '':
                schema['module_id'] = model['module_id']

            pkg = list(filter(lambda x: x['id'] ==
                       model['package_id'], result['packages']))[0]

            mod = list(filter(lambda x: x['id'] ==
                       model['module_id'], result['modules']))[0]

            key = ('' if pkg['id'] == 0 else pkg['name'] + '.') + \
                mod['name'] + '.' + ('State' if model['name']
                                     == '' else model['name'])
            schemas[key] = schema

            id_key = '-'.join(['model', str(pkg['id']),
                              str(mod['id']), str(model['id'])])
            schemas[id_key] = schema

        hfn = list(filter(
            lambda x: x['schema_id'] == schema['id'] and x['package_id'] == schema['pkg_id'], result['hfns']))

        if len(hfn) > 0:
            hfn = hfn[0]
            schema['hfn_id'] = hfn['id']

            pkg = list(filter(lambda x: x['id'] ==
                       hfn['package_id'], result['packages']))[0]

            mod = list(filter(lambda x: x['id'] ==
                       hfn['module_id'], result['modules']))[0]

            key = ('' if pkg['id'] == 0 else pkg['name'] + '.') + \
                mod['name'] + '.' + hfn['name']

            schemas[key] = schema

            id_key = '-'.join(['hfn', str(pkg['id']),
                              str(mod['id']), str(hfn['id'])])
            schemas[id_key] = schema

        schemas[str(schema['pkg_id']) + '-' + str(schema['id'])] = schema

    hfn_core.run()

    executor = ThreadPoolExecutor(kwargs.get('max_workers'))

    def event_loop():
        while True:
            data = hfn_core.read()
            unpacker = msgpack.Unpacker()
            unpacker.feed(bytes(data))

            [pkg_id, headers, payload, socket_id] = list(unpacker)

            unpacker = msgpack.Unpacker()
            unpacker.feed(bytes(payload))
            msg = list(unpacker)

            if msg[0] == 1:
                _, module_id, hfn_id, cookies, data = msg

                handler_id = '-'.join([str(pkg_id),
                                      str(module_id), str(hfn_id)])
                schema = schemas.get('hfn-' + handler_id)
                if not schema:
                    continue

                handler = handlers.get(handler_id)
                if not handler:
                    continue

                model = Model(schema, schemas)

                model.decode(data)

                ctx = Context(
                    package_id=pkg_id,
                    socket_id=socket_id,
                    headers=headers,
                    cookies=cookies,
                    data=model,
                    module_id=module_id,
                    hfn_id=hfn_id,
                    schemas=schemas
                )

                executor.submit(handler, (ctx))

    task = executor.submit(event_loop)
    task.result()
