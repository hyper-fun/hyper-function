class HomeViewModule:
    name = 'hahah'

    def show(self, ctx):
        print('HomeViewModule.show ' + ctx)

    def hide(ctx):
        print('HomeViewModule.hide')


def get_methods(c):
    return list(filter(lambda x: not x.startswith('__') and callable(getattr(c, x)), dir(c)))


print(HomeViewModule.__name__)
print(get_methods(HomeViewModule))

homeView = HomeViewModule()
fn = getattr(homeView, 'show')
fn('ahaha')
