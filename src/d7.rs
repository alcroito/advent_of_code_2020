use advent::helpers;
use anyhow::{Context, Result};
use petgraph::graphmap::DiGraphMap;
use petgraph::visit::{Dfs, DfsPostOrder, Reversed, Walker};
use std::collections::HashMap;

type NodeName<'a> = &'a str;
type BagGraph<'a> = DiGraphMap<NodeName<'a>, u32>;
type BagCount = u32;
type NodeBagCounter<'a> = HashMap<NodeName<'a>, BagCount>;

fn str_to_graph(input: &str) -> BagGraph {
    let mut graph = BagGraph::new();
    let bag_relations = input
        .trim()
        .lines()
        .map(|l: &str| {
            let l = l.trim();
            let contain_token = " contain ";
            let name_index_end = l.find(contain_token).expect("no contain token found");
            let bag_name = l
                .get(0..name_index_end)
                .expect("no bag name found")
                .trim_end_matches('s');
            let relations_start_index = name_index_end + contain_token.len();
            let relations = l
                .get(relations_start_index..l.len())
                .expect("no relations found");
            let relations = relations
                .trim_end_matches('.')
                .split(", ")
                .map(|count_and_name| {
                    if count_and_name.contains("no other bags") {
                        None
                    } else {
                        let first_space_index = count_and_name
                            .find(' ')
                            .expect("no space between count and name");
                        let count_and_name = (
                            count_and_name
                                .get(0..first_space_index)
                                .expect("No count found")
                                .parse::<u32>()
                                .expect("Invalid count"),
                            count_and_name
                                .get(first_space_index + 1..count_and_name.len())
                                .expect("No name found")
                                .trim_end_matches('s'),
                        );
                        Some(count_and_name)
                    }
                })
                .collect::<Vec<_>>();
            (bag_name, relations)
        })
        .collect::<Vec<_>>();
    bag_relations.iter().for_each(|(bag_name, _)| {
        graph.add_node(bag_name);
    });
    bag_relations.iter().for_each(|(bag_name, relations)| {
        relations.iter().for_each(|maybe_contain_relation| {
            if let Some((count, other_bag_name)) = maybe_contain_relation {
                graph.add_edge(bag_name, other_bag_name, *count);
            }
        });
    });
    // println!("{:?}", graph);
    graph
}

fn compute_bag_color_count_containing_gold(g: &BagGraph) -> u32 {
    let dfs = Dfs::new(g, "shiny gold bag").iter(Reversed(g));
    (dfs.count() - 1) as u32
}

fn compute_gold_bag_required_bag_count(g: &BagGraph) -> u32 {
    let counter = g
        .nodes()
        .into_iter()
        .map(|bag_name| (bag_name, 0))
        .collect::<NodeBagCounter>();
    let initial_node = "shiny gold bag";
    let dfs = DfsPostOrder::new(g, initial_node);
    let counter = dfs.iter(&g).fold(counter, |mut counter, current_bag| {
        let current_bag_count: BagCount = g
            .neighbors_directed(current_bag, petgraph::Direction::Outgoing)
            .map(|contained_bag| {
                let contained_bag_count = g
                    .edge_weight(current_bag, contained_bag)
                    .expect("Non-existent edge");
                let contained_bag_inner_count = counter[contained_bag];
                contained_bag_count + contained_bag_count * contained_bag_inner_count
            })
            .sum();
        counter
            .entry(current_bag)
            .and_modify(|e| *e = current_bag_count);
        // println!("visiting: {}, required count: {}", current_bag, current_bag_count);
        counter
    });
    counter[initial_node]
}

fn solve_p1() -> Result<()> {
    let data = helpers::get_data_from_file_res("d7").context("Coudn't read file contents.")?;
    let g = str_to_graph(&data);
    let count = compute_bag_color_count_containing_gold(&g);
    println!(
        "Bag color count that can contain shiny gold bags: {}",
        count
    );
    Ok(())
}

fn solve_p2() -> Result<()> {
    let data = helpers::get_data_from_file_res("d7").context("Coudn't read file contents.")?;
    let g = str_to_graph(&data);
    let count = compute_gold_bag_required_bag_count(&g);
    println!("Shiny gold bags need to contain this many bags: {}", count);
    Ok(())
}

fn main() -> Result<()> {
    solve_p1().ok();
    solve_p2()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p1() {
        let input = "
        light red bags contain 1 bright white bag, 2 muted yellow bags.
        dark orange bags contain 3 bright white bags, 4 muted yellow bags.
        bright white bags contain 1 shiny gold bag.
        muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.
        shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.
        dark olive bags contain 3 faded blue bags, 4 dotted black bags.
        vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.
        faded blue bags contain no other bags.
        dotted black bags contain no other bags.";
        let g = str_to_graph(input);
        let count = compute_bag_color_count_containing_gold(&g);
        assert_eq!(count, 4);
    }

    #[test]
    fn test_p2() {
        let input = "
        light red bags contain 1 bright white bag, 2 muted yellow bags.
        dark orange bags contain 3 bright white bags, 4 muted yellow bags.
        bright white bags contain 1 shiny gold bag.
        muted yellow bags contain 2 shiny gold bags, 9 faded blue bags.
        shiny gold bags contain 1 dark olive bag, 2 vibrant plum bags.
        dark olive bags contain 3 faded blue bags, 4 dotted black bags.
        vibrant plum bags contain 5 faded blue bags, 6 dotted black bags.
        faded blue bags contain no other bags.
        dotted black bags contain no other bags.";
        let g = str_to_graph(input);
        let count = compute_gold_bag_required_bag_count(&g);
        assert_eq!(count, 32);

        let input = "
        shiny gold bags contain 2 dark red bags.
        dark red bags contain 2 dark orange bags.
        dark orange bags contain 2 dark yellow bags.
        dark yellow bags contain 2 dark green bags.
        dark green bags contain 2 dark blue bags.
        dark blue bags contain 2 dark violet bags.
        dark violet bags contain no other bags.";
        let g = str_to_graph(input);
        let count = compute_gold_bag_required_bag_count(&g);
        assert_eq!(count, 126);
    }
}
