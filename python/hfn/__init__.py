from package import Package
from hfn import run


class HomeView:
    name = 'hahah'

    def mount(self, ctx):
        print('HomeViewModule.mount ' + ctx)

    def hide(ctx):
        print('HomeViewModule.hide')


packages = Package([HomeView])

run([packages], dev=False)
