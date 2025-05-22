import "tetron:log" as log;
import "tetron:flags" as flags;
// import "tetron:behaviours" as behaviours;

let vec2 = Vec2::create(4, 5);

log::info(`${vec2.x}, ${vec2.y}`);

let world = World::create();
let scene = Scene::create(#{
    bg: "#120321",
    gravity: vec2(0, 0)
});

fn Paddle(x, y) {
    Entity::create()
        .attach(behaviours::drawable::Rect("#007cdf", #{ w: 80, h: 10 }))
        .attach({
            let p = behaviours::Physics::rect(#{ w: 80, h: 10, pos: vec2(x, y), friction: 0.8, collision: "immovable" });
            p.friction = 0.8;
            p
        })
        .attach({
            let input = behaviours::Input::create();
            input.on("update", |ctx, _dt, inputs| {
                let physics = ctx.behaviour("physics");
                if (inputs.key("right")) {
                    physics.apply_force(vec2(1, 0));
                } else if (inputs.key("left")) {
                    physics.apply_force(vec2(-1, 0));
                } else {
                    physics.vel.x = 0;
                }
            });
            input
        })
}

fn Ball(x, y, vx, vy) {
    Entity::create()
        .attach(behaviours::drawable::Circle("circle", "#ffffff", #{ r: 5 }))
        .attach({
            let phys = behaviours::Physics::circle(#{ r: 5, collision: "physical", pos: vec2(x, y) });
            phys.vel = vec2(vx, vy);
            phys
        })
        .tag("ball")
}

fn Wall(x, y, w, h) {
    Entity::create()
        .attach(behaviours::drawable::Rect("#333", #{ w: w, h: h }))
        .attach({
            let phys = behaviours::Physics::rect(#{ w: w, h: h, pos: vec2(x, y) });
            phys.immovable = true;
            phys
        })
}

fn create_brick(x, y) {
    Entity::create()
        .attach(behaviours::drawable::Rect("#ff6600", #{ w: 60, h: 20 }))
        .attach({
            let phys = behaviours::Physics::rect(#{ w: 60, h: 20, pos: vec2(x, y) });
            phys.immovable = true;
            phys.on("collide", |ctx, other| {
                if (other.has_tag("ball")) {
                    ctx.destroy();
                }
            });
            phys
        })
}

// Setup
scene.insert_many([
    Paddle(320, 460),
    Ball(320, 240, 2, -3),
    Wall(0, 0, 640, 10),
    Wall(0, 0, 10, 480),
    Wall(630, 0, 10, 480)
]);

for row in 0..5 {
    for col in 0..10 {
        let x = 20 + col * 62;
        let y = 40 + row * 22;
        scene.insert(create_brick(x, y));
    }
}

scene.on("update", |ctx, dt| {
    for ball in ctx.scene().find_by_tag("ball") {
        if (ball.position().y > 480) {
            log::info("Ball fell off the screen!");
            // todo: game over logic
        }
    }
});

world.insert(scene);
