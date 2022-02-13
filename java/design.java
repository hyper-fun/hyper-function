import java.lang.reflect.Method;

/**
 * e
 */
public class E {

  public static void main(String[] args) {
    System.out.println(HomeViewModule.name);

    Method[] m = HomeViewModule.class.getDeclaredMethods();
    for (int i = 0; i < m.length; i++) {
      System.out.println(m[i].getName());
    }

    HomeViewModule homeView = new HomeViewModule();
    try {
      Method hfn = HomeViewModule.class.getMethod("show", Context.class);
      hfn.invoke(homeView, new Context("blalala"));
    } catch (Exception e) {
      // TODO: handle exception
      System.out.println(e);
    }
  }
}

class Context {
  final String name;

  public Context(String name) {
    this.name = name;
    System.out.println("Context init");
  }
}

class HomeViewModule {
  static String name = "efsef";

  public void show(Context ctx) {
    System.out.println("HomeView " + ctx.name);
  }
}
