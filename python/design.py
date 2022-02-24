class HomeView:
    name = 'eee'
    age = '2'

    def __init__(self):
        print('HomeView.__init__')
        self.age = '18'

    def show(self, ctx):
        print('HomeView.show ' + self.name + ctx + ':' + self.age)

    def hide(ctx):
        print('HomeView.hide')


homeView = HomeView.__new__(HomeView)
homeView.__init__()
homeView.show('fwefwe')
fn = getattr(homeView, 'show')
fn('ahaha')

print('aasss' in ['aass'])
print(isinstance(b'ababa', bytes))
