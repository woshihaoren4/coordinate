use crate::app::entity::{NodeEntity, TaskSlot};
use std::collections::HashSet;

pub struct SoltRebalance<'a> {
    slot: &'a TaskSlot,
    nodes: Vec<NodeEntity>,
    // des : NodeEntity,
}

impl SoltRebalance<'_> {
    pub fn new(task: &TaskSlot, nodes: Vec<NodeEntity>) -> SoltRebalance {
        SoltRebalance { slot: task, nodes }
    }
    pub fn balance(self) -> Vec<NodeEntity> {
        if self.nodes.len() <= 0 {
            return vec![];
        }
        let slot = self.slot;
        let ns = self.nodes;

        let mut tags_set = HashSet::new();
        for i in 0..slot.slot_count {
            tags_set.insert(i);
        }
        let mut list = vec![];
        let mut exp = slot.slot_count / ns.len() as i32; //期望平均数量
        let mut more = slot.slot_count % ns.len() as i32; //比期望多的部分
        if more != 0 {
            exp += 1;
        } else {
            more = ns.len() as i32;
        }
        let max = if slot.node_max_count > exp {
            exp
        } else {
            slot.node_max_count
        };

        //找出多余的标签
        for mut i in ns.into_iter() {
            if i.tags.len() as i32 > exp {
                let (other, _) = i.tags.split_at(exp as usize);
                i.tags = other.to_vec();
            }
            for j in i.tags.iter() {
                tags_set.remove(j);
            }
            if i.tags.len() as i32 == exp {
                list.insert(0, i);
            } else {
                list.push(i);
            }
        }
        //将多余的标签分配给所有节点
        for i in list.iter_mut() {
            if i.tags.len() as i32 > max {
                //多于应该则删除
                let index = if more > 0 {
                    more -= 1;
                    max
                } else {
                    max - 1
                };
                let (other, rm) = i.tags.split_at(index as usize);
                println!("other->{:?}", other);
                println!("other->{:?}", rm);
                for j in rm.iter() {
                    tags_set.insert(*j);
                }
                i.tags = other.to_vec();
            } else if i.tags.len() as i32 == max {
                if more > 0 {
                    more -= 1;
                } else {
                    if let Some(j) = i.tags.pop() {
                        tags_set.insert(j);
                    }
                }
            } else {
                let mut list = vec![];
                for j in tags_set.iter() {
                    i.tags.push(*j);
                    list.push(*j);
                    let limit = if more > 0 { max } else { max - 1 };
                    if i.tags.len() as i32 >= limit {
                        if more > 0 {
                            more -= 1;
                        }
                        break;
                    }
                }
                for j in list.iter() {
                    tags_set.remove(j);
                }
            }
        }

        return list;
    }
    pub fn join(mut self, des: NodeEntity) -> Vec<NodeEntity> {
        //是否已经加入
        for i in self.nodes.iter() {
            if i.code == des.code {
                return self.nodes;
            }
        }
        self.nodes.push(des);
        return self.balance();
    }
    pub fn remove(mut self, code: String) -> Vec<NodeEntity> {
        //是否已经加入
        let mut i = None;
        {
            for (index, n) in self.nodes.iter().enumerate() {
                if n.code == code {
                    i = Some(index);
                    break;
                }
            }
        }
        match i {
            None => {
                return self.nodes;
            }
            Some(i) => {
                self.nodes.remove(i);
            }
        }
        return self.balance();
    }
}

#[cfg(test)]
mod test {
    use crate::app::entity::{NodeEntity, TaskEntity, TaskSlot};
    use crate::app::service::rebalance::SoltRebalance;
    use std::collections::HashMap;

    #[test]
    fn test_join_one() {
        let slot = TaskSlot {
            slot_count: 32,
            node_max_count: 8,
            node_min_count: 0,
        };
        let node_one = NodeEntity {
            tags: vec![],
            ..Default::default()
        };

        let result = SoltRebalance::new(&slot, vec![node_one]).balance();
        println!("{:?}", result)
    }

    #[test]
    fn test_join_two() {
        let slot = TaskSlot {
            slot_count: 32,
            node_max_count: 16,
            node_min_count: 0,
        };
        let node_one = NodeEntity {
            tags: vec![17, 9, 4, 21, 8, 16, 31],
            ..Default::default()
        };
        let node_two = NodeEntity {
            tags: vec![],
            ..Default::default()
        };

        let result = SoltRebalance::new(&slot, vec![node_one]).join(node_two);
        println!("{:?}", result)
        //17, 9, 4, 21, 8, 16, 31, 28, 25, 23, 30, 12, 11, 24, 1
        //15, 6, 13, 3, 5, 20, 7, 0, 2, 29, 10, 14, 27, 19, 22
    }

    #[test]
    fn test_join_three() {
        let slot = TaskSlot {
            slot_count: 32,
            node_max_count: 16,
            node_min_count: 0,
        };

        let node_two = NodeEntity {
            tags: vec![15, 6, 13, 3, 5, 20, 7, 0, 2, 29, 19],
            ..Default::default()
        };
        let node_three = NodeEntity {
            tags: vec![26, 10, 1, 11, 24, 14, 18, 12, 27, 22],
            ..Default::default()
        };
        let node_one = NodeEntity {
            tags: vec![17, 9, 4, 21, 8, 16, 31, 28, 25, 23, 30],
            ..Default::default()
        };

        let result = SoltRebalance::new(&slot, vec![node_one, node_two, node_three]).balance();
        println!("{:?}", result)
        //17, 9, 4, 21, 8, 16, 31, 28, 25, 23, 30
        //15, 6, 13, 3, 5, 20, 7, 0, 2, 29, 19
        //26, 10, 1, 11, 24, 14, 18, 12, 27, 22
    }
    #[test]
    fn test_join_four() {
        let slot = TaskSlot {
            slot_count: 32,
            node_max_count: 16,
            node_min_count: 0,
        };
        let node_one = NodeEntity {
            tags: vec![17, 9, 4, 21, 8, 16, 31, 28, 25, 23, 30, 12, 11, 24, 1],
            ..Default::default()
        };
        let node_two = NodeEntity {
            tags: vec![15, 6, 13, 3, 5, 20, 7, 0, 2, 29],
            ..Default::default()
        };
        let node_three = NodeEntity {
            tags: vec![],
            ..Default::default()
        };
        let node_four = NodeEntity {
            tags: vec![26, 10, 1, 11, 24, 14, 18, 12],
            ..Default::default()
        };

        let result =
            SoltRebalance::new(&slot, vec![node_one, node_two, node_four]).join(node_three);
        println!("{:?}", result)
        //17, 9, 4, 21, 8, 16, 31, 28, 25, 23, 30
        //15, 6, 13, 3, 5, 20, 7, 0, 2, 29, 19
        //26, 10, 1, 11, 24, 14, 18, 12, 27, 22
    }
}
