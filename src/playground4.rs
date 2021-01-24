/*
1) Are the 'a and 'static bounds applied specifically to any possible references held that might have been captured by the closures?
   yes
   'a means captures are either owned or live at least 'a
   'static means captures are either owned or live as long as the program does
2) Does there have to be any relation between the lifetime of the input/output argument of the closure, and the lifetime bound on the closure?
   No, but there can be.
3) If the closures don't capture any outside state, does that mean the trait bounds are superflous?
   Yep.
4) Is there any difference between f1 and f3, aka between a Boxed clsoure and a ref to a closure?
   The only differences are the storage (stack vs heap), and the implicit trait object lifetimes ('static for Box, 'a for ref).
4.1) Would the only difference be the default implicit lifetime bounds if i hadn't explicitly set them? e.g. "'a" for closure ref, "'static" for boxed closure?
   Yep
5) Does MyFun5/6 make any sense?
   Yep. Depends on the specific case.
6) Does a 'static bound on the FnMut mean that all refs held by it have to be alive for the whole program execution or does it mean that the lifetimes are unbound, and can live for whatever lifetime is necessary?
   Nope, see point 1)
7) when to use HRTBs
   No idea yet, will have to bump into it eventually.
   type Foo = Box<dyn for<'a> FnMut(&'a str) -> &'a str>;
   fn main() {}
   https://users.rust-lang.org/t/understanding-lifetimes-of-generic-fn-trait-objects/25255
*/

type In<'a> = &'a str;
type MyFun1<'a> = dyn FnMut(In<'a>) -> In<'a> + 'a;
type MyFun2<'a> = dyn FnMut(In<'a>) -> In<'a> + 'static;

type MyFun3<'a> = Box<dyn FnMut(In<'a>) -> In<'a> + 'a>;
type MyFun4<'a> = Box<dyn FnMut(In<'a>) -> In<'a> + 'static>;

type MyFun5<'a, 'b> = dyn FnMut(In<'a>) -> In<'a> + 'b;
type MyFun6<'a, 'b> = Box<dyn FnMut(In<'a>) -> In<'a> + 'b>;

fn main() {
    let s_owned = "s".to_owned();
    let s = &s_owned;
    let s_static: &'static str = "static";
    let mut state1 = 0usize;
    let mut state3 = 0usize;
    let mut state5 = 0usize;
    let mut state6 = 0usize;

    let f1: &mut MyFun1 = &mut |a:&str| {state1+=1; a};
    let f2: &mut MyFun2 = &mut |a:&str| {a};
    let mut f3: MyFun3 = Box::new(|a:&str| {state3+=1; a});
    let mut f4: MyFun4 = Box::new(|a:&str| {a});
    let f5: &mut MyFun5 = &mut |a:&str| {state5+=1; a};
    let mut f6: MyFun6 = Box::new(|a:&str| {state6+=1; a});

    f1(s);
    f1(s_static);

    f2(s);
    f2(s_static);

    f3(s);
    f3(s_static);

    f4(s);
    f4(s_static);

    f5(s);
    f5(s_static);

    f6(s);
    f6(s_static);
}