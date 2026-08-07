---
name: bevy-queries
description: Reference for writing Bevy ECS queries — Query<D, F>, Single, filters, component access, iteration patterns, lenses, disjointed access, and testing.
metadata:
  crate: bevy_ecs
  bevy: "0.19"
---

## Basics

`Query<D, F>` is a system parameter. `D` = query data (what to fetch), `F` = query filter (conditions). Query items are fetched when iterating or calling a getter.

```rust
fn system(query: Query<&Transform>) {
  for transform in &query { }
}
```

## Component mutability

| Syntax | Meaning |
|--------|---------|
| `Query<&T>` | Readonly borrow — parallel-friendly |
| `Query<&mut T>` | Mutable borrow — conflicts with other access to T unless Bevy can prove the queries are disjoint |
| `Query<Option<&T>>` | Optional component — entity may or may not have it |

## Tuple = AND logic

Each generic parameter in `Query<D, F>` can be a tuple. Query-data tuples require every non-optional item; filter tuples apply every filter.

```rust
Query<(&Ball, &Player)>                // entities with both Ball AND Player
Query<&Transform, (With<Player>, With<Living>)>  // Transform on entities with Player AND Living
```

## Alternative query parameters

| Parameter | Behavior |
|-----------|----------|
| `Single<D, F>` | Exactly one match, or system skipped |
| `Option<Single<D, F>>` | Zero or one match |
| `Populated<D, F>` | One or more matches, skips if none |

```rust
fn move(mut t: Single<&mut Transform, With<Player>>) {
  t.translation.x += 1.;
}
fn destructure(s: Single<(&mut Pos, &Vel), With<Player>>) {
  let (mut pos, vel) = s.into_inner();
}
```

## QueryData (D parameter)

| Type | Description |
|------|-------------|
| `&T` / `&mut T` | Read or write component |
| `Option<D>` | Optional query data; does not require D to match |
| `AnyOf<T>` | Fetch entities matching any of the tuple types |
| `Ref<T>` | Readonly with change detection methods |
| `Has<T>` | Returns whether the entity has T; does not filter entities |
| `Entity` | The entity ID |

### AnyOf

```rust
Query<AnyOf<(&Player, &Rocket, &mut Asteroid)>>
// Expands to: Query<(Option<&P>, Option<&R>, Option<&mut A>), Or<(With<P>, With<R>, With<A>)>>
```

### Ref (change detection)

```rust
fn check(q: Query<Ref<Player>>) {
  for p in &q {
    if p.is_added() { }
    if p.is_changed() { }
    // p.last_changed()
  }
}
```

### Entity ID

```rust
fn with_id(q: Query<(Entity, &Transform)>) {
  for (entity, transform) in &q { }
}
fn lookup(players: Query<Entity, With<Player>>, transforms: Query<&Transform>) {
  for e in &players {
    let t = transforms.get(e).unwrap();
  }
}
```

## QueryFilter (F parameter)

| Filter | Description |
|--------|-------------|
| `With<T>` | Only entities with component T |
| `Without<T>` | Only entities without component T |
| `Or<F>` | Union of filters in tuple |
| `Changed<T>` | Component changed since this system last ran |
| `Added<T>` | Component added since this system last ran |

```rust
Query<&Transform, (With<Player>, Without<Dead>)>
Query<&Player, Added<Player>>  // equivalent to filtering Ref<Player> with is_added()
```

## Retrieval methods

| Method | Description |
|--------|-------------|
| `iter` / `iter_mut` | Iterator over all matches |
| `iter().for_each` | Sequential iterator closure; may optimize better than a for loop |
| `iter_many` / `iter_many_mut` | Iterate over specific entity list |
| `iter_combinations` | All K-combinations of matches |
| `par_iter` / `par_iter_mut` | Parallel iteration |
| `get` / `get_mut` | Fetch single entity's components by Entity |
| `get_many` / `get_many_mut` | Fetch a fixed array of entities; returns `Result` |
| `single` / `single_mut` | Fetch exactly one match; returns `Result` |
| `is_empty` | Check if query has matches |
| `contains` | Check if query contains specific entity |

```rust
// Iteration
for mut t in &mut query { }
query.iter_mut().for_each(|mut t| { });

// Specific entity
if let Ok(t) = query.get(entity) { }

// Combinations
for [a, b] in query.iter_combinations() { }

// Many entities
let mut iter = query.iter_many_mut(&entities);
while let Some(mut h) = iter.fetch_next() { }
```

## Query lenses

Share common query logic without duplicating system parameters.

```rust
fn print_health(lens: &mut QueryLens<&Health>) {
  for h in &mut lens.query() {
    if h.0 > 50.0 { info!("healthy"); }
  }
}
fn player_system(mut q: Query<(&Health, &Player)>) {
  print_health(&mut q.transmute_lens::<&Health>());
}
fn enemy_system(mut q: Query<(&Health, &Enemy, &Transform)>) {
  print_health(&mut q.transmute_lens::<&Health>());
}
```

- `transmute_lens` — narrow query data
- `transmute_lens_filtered` — include filter
- `join` / `join_filtered` — combine queries

## Disjointed queries & ParamSet

Two queries with mutable access to overlapping component sets: use `Without` to make them disjoint, or use `ParamSet` so Rust permits only one contained parameter borrow at a time.

```rust
// Will panic at runtime — ambiguous borrows
fn bad(p: Query<&mut Player, With<Rocket>>, e: Query<&mut Player, With<Invincibility>>) { }

// Safe: serialized access
fn ok(p: ParamSet<(Query<&mut Player, With<Rocket>>, Query<&mut Player, With<Invincibility>>)>) { }

// Or with disjoint archetypes (Bevy 0.12+):
fn disjoint(t: Query<EntityMut, With<Transform>>, e: Query<EntityMut, Without<Transform>>) { }
```

## Performance notes

- `Table` storage iterates faster than `SparseSet`
- Two systems with conflicting mutable access to the same component type cannot run in parallel
- `Iterator::for_each` may outperform a `for` loop on worlds with high archetype fragmentation
- Prefer a `for` loop unless profiling shows `for_each` helps
- `entity.get_components_mut::<(&mut A, &mut B)>()` checks requested components for conflicts in O(n²)

## Testing

Use `app.world_mut().run_system_once(fn)` or access `app.world_mut().query::<T>()`:

```rust
#[cfg(test)]
mod tests {
  use super::*;

  fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, plugin));
    app
  }

  fn check_ship(query: Query<&Ship>) {
    assert_eq!(query.iter().count(), 1);
  }

  #[test]
  fn test_spawn() {
    let mut app = setup_app();
    app.update();
    app.world_mut().run_system_once(check_ship).unwrap();
  }
}
```
