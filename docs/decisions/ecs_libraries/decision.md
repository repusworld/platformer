# ECS decision

In this document I want to document which ecs was chosen and why.

## Libraries I tested
- legion
- hecs
- shipyard

## Legion
Legion was pretty easy to get started.

## hecs
hecs is almost the same as legion, syntax wise, but it's a bit easier to use. hecs is also more lightweight.

## Shipyard
Shipyard is a mess. Almost everything needs way more code than with legion or hecs.
Also to do anything, you need to pass a lambda to the run function, which has its own problems.

The only pro I found in this short test is, that it's easier to iterate over multiple archetypes at the same time (e.g.: all cameras and all players).

## Performance
In this small example all libraries performed roughly the same, but for some strange reason every version runs slower when built in release mode (~2000fps instead of 2000-3000fps)

## Conclusion
It was a pretty close call between hecs and legion, but for now I'm going with hecs because it's a bit simpler and more lightweight.
Also, because those two are so similar, I think it would be easy to change them in the future if the need arises.

### Here are the diffs I used to compare the complexity of the libraries
- [hecs vs legion](hecs_vs_legion.md)
- [hecs vs shipyard](hecs_vs_shipyard.md)