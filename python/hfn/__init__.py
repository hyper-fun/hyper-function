from package import Package
from hfn import run


class HomeView:
    def mount(self, ctx):
        data = ctx.data.to_dict()
        state = ctx.model('homeView.State')
        state.set('str', '123')
        state.set('strArr', ['123', '234', '235'])
        state.set('int', 123)
        state.set('intArr', [123, 234, 235])
        state.set('float', 1.2)
        state.set('floatArr', [2.3, 4.5, 6.5])
        state.set('bool', True)
        state.set('boolArr', [False, True, False])
        state.set('bytes', b'123')
        state.set('bytesArr', [b'123', b'234', b'235'])

        nested = ctx.model('homeView.ahaha')
        nested.set("id", 2323)
        nested.set("s", "baba")
        state.set("nested", nested)
        state.set("nestedArr", [nested, nested, nested])

        ctx.set_state(state)

    def hide(ctx):
        print('HomeViewModule.hide')


packages = Package([HomeView])

run([packages], dev=True)
