use std::collections::BTreeMap;
use std::sync::Mutex;

use super::*;

type ValueMap = BTreeMap<Value, Value>;

#[derive(Debug)]
pub struct MapSort {
    name: Symbol,
    key: ArcSort,
    value: ArcSort,
    maps: Mutex<IndexSet<ValueMap>>,
}

impl MapSort {
    fn kv_names(&self) -> (Symbol, Symbol) {
        (self.key.name(), self.value.name())
    }

    pub fn make_sort(egraph: &mut EGraph, name: Symbol, args: &[Expr]) -> Result<ArcSort, Error> {
        if let [Expr::Var(k), Expr::Var(v)] = args {
            let k = egraph.sorts.get(k).ok_or(TypeError::UndefinedSort(*k))?;
            let v = egraph.sorts.get(v).ok_or(TypeError::UndefinedSort(*v))?;
            Ok(Arc::new(Self {
                name,
                key: k.clone(),
                value: v.clone(),
                maps: Default::default(),
            }))
        } else {
            panic!()
        }
    }
}

impl Sort for MapSort {
    fn name(&self) -> Symbol {
        self.name
    }

    fn as_arc_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync + 'static> {
        self
    }

    fn register_primitives(self: Arc<Self>, egraph: &mut EGraph) {
        egraph.add_primitive(Ctor {
            name: "empty".into(),
            map: self.clone(),
        });
        egraph.add_primitive(Insert {
            name: "insert".into(),
            map: self.clone(),
        });
        egraph.add_primitive(Get {
            name: "get".into(),
            map: self,
        });
    }

    fn make_expr(&self, value: Value) -> Expr {
        let map = ValueMap::load(self, &value);
        let mut expr = Expr::call("empty", []);
        for (k, v) in map.iter().rev() {
            let k = self.key.make_expr(*k);
            let v = self.value.make_expr(*v);
            expr = Expr::call("insert", [expr, k, v])
        }
        expr
    }
}

impl IntoSort for ValueMap {
    type Sort = MapSort;
    fn store(self, sort: &Self::Sort) -> Option<Value> {
        let mut maps = sort.maps.lock().unwrap();
        let (i, _) = maps.insert_full(self);
        Some(Value {
            tag: sort.name,
            bits: i as u64,
        })
    }
}

impl FromSort for ValueMap {
    type Sort = MapSort;
    fn load(sort: &Self::Sort, value: &Value) -> Self {
        let maps = sort.maps.lock().unwrap();
        maps.get_index(value.bits as usize).unwrap().clone()
    }
}

struct Ctor {
    name: Symbol,
    map: Arc<MapSort>,
}

impl PrimitiveLike for Ctor {
    fn name(&self) -> Symbol {
        self.name
    }

    fn accept(&self, types: &[&dyn Sort]) -> Option<ArcSort> {
        match types {
            [] => Some(self.map.clone()),
            _ => None,
        }
    }

    fn apply(&self, values: &[Value]) -> Option<Value> {
        assert!(values.is_empty());
        ValueMap::default().store(&self.map)
    }
}

struct Insert {
    name: Symbol,
    map: Arc<MapSort>,
}

impl PrimitiveLike for Insert {
    fn name(&self) -> Symbol {
        self.name
    }

    fn accept(&self, types: &[&dyn Sort]) -> Option<ArcSort> {
        match types {
            [map, key, value]
                if (map.name(), (key.name(), value.name()))
                    == (self.map.name, self.map.kv_names()) =>
            {
                Some(self.map.clone())
            }
            _ => None,
        }
    }

    fn apply(&self, values: &[Value]) -> Option<Value> {
        let mut map = ValueMap::load(&self.map, &values[0]);
        map.insert(values[1], values[2]);
        map.store(&self.map)
    }
}

struct Get {
    name: Symbol,
    map: Arc<MapSort>,
}

impl PrimitiveLike for Get {
    fn name(&self) -> Symbol {
        self.name
    }

    fn accept(&self, types: &[&dyn Sort]) -> Option<ArcSort> {
        match types {
            [map, key] if (map.name(), key.name()) == (self.map.name, self.map.key.name()) => {
                Some(self.map.value.clone())
            }
            _ => None,
        }
    }

    fn apply(&self, values: &[Value]) -> Option<Value> {
        let map = ValueMap::load(&self.map, &values[0]);
        map.get(&values[1]).copied()
    }
}
