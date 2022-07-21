use std::collections::BTreeSet;

use flexlayout::{Context, Id, Layout, Scalar};
use macroquad::prelude::*;

#[macroquad::main("2d showcase")]
async fn main() {
    loop {
        let (ctx, leaves) = gen_ui();

        clear_background(LIGHTGRAY);

        for &el in &leaves {
            draw_item(&ctx, el, WHITE);
        }
        next_frame().await;
    }

    // for el in &leaves {
    //     let a = ctx.item_rect_err(el).expect("no id");
    //     println!("{:?}", a);
    // }

    // sleep
    // for k in std::io::stdin().lines() {}
}

fn gen_ui() -> (Context<2>, BTreeSet<Id>) {
    let mut ctx = Context::<2>::new();
    let root = ctx.item_new_mut(|item| {
        item.size = [
            Some(screen_width() as Scalar),
            Some(screen_height() as Scalar),
        ];
        item.flags.as_parent.layout = Layout::Flex(1.try_into().unwrap());
        item.flags.as_parent.alignment_along_axis.front = true;
        item.flags.as_parent.alignment_along_axis.back = true;
    });
    let mut leaves: BTreeSet<Id> = unsafe { std::mem::zeroed() };
    let mut rows = vec![];
    for (row_len, align_front, align_back) in [
        (6, true, true),
        (7, false, false),
        (8, true, false),
        (9, false, true),
    ] {
        let row = ctx.item_new_mut(|item| {
            item.size = [None, None];
            item.flags.as_parent.layout = Layout::Flex(0.try_into().unwrap());
            item.margins[1].start = 20;
            item.margins[1].end = 20;
            item.flags.as_parent.alignment_along_axis.front = align_front;
            item.flags.as_parent.alignment_along_axis.back = align_back;
            item.flags.as_child.alignment_cross_axis[0].front = true;
            item.flags.as_child.alignment_cross_axis[0].back = true;
        });
        rows.push(row);
        ctx.push_back(root, row).unwrap();
        for i in 0..row_len {
            let child = ctx.item_new_mut(|item| {
                item.size = [Some(20), Some(20 + i * 5)];
                item.margins[0].start = 10;
                item.margins[0].end = 10;
                item.flags.as_child.alignment_cross_axis[1].front = align_front;
                item.flags.as_child.alignment_cross_axis[1].back = align_back;
            });

            ctx.push_back(row, child).unwrap();
            leaves.insert(child);
        }
    }
    ctx.layout_item_recursively(root).expect("layout failed");

    // println!("root  {:?}", ctx.item_rect_err(root).unwrap());
    // println!("row0  {:?}", ctx.item_rect_err(rows[1]).unwrap());
    // println!("row1  {:?}", ctx.item_rect_err(rows[0]).unwrap());

    (ctx, leaves)
}

fn draw_item<const ND: usize>(ctx: &Context<ND>, root: flexlayout::Id, color: Color) {
    let a = ctx.item_rect_err(root).expect("no id");
    draw_rectangle(
        a.position[0] as f32,
        a.position[1] as f32,
        a.size[0] as f32,
        a.size[1] as f32,
        color,
    );
}
