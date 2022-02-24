class Package:
    name = ""
    modules = {}

    def __init__(self, modules, **kwargs):
        name = kwargs.get('name')
        if name:
            self.name = kwargs.name

        for module in modules:
            name = module.__name__
            methods = get_methods(module)
            instance = module.__new__(module)
            instance.__init__()

            self.modules.setdefault(name.lower(), {
                'name': name,
                'methods': methods,
                'instance': instance
            })


def get_methods(c):
    return list(filter(lambda x: not x.startswith('__') and callable(getattr(c, x)), dir(c)))
