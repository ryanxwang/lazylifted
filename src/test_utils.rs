pub const BLOCKSWORLD_DOMAIN_TEXT: &str = r#"
(define (domain blocksworld)
        
(:requirements :strips)

(:predicates (clear ?x)
             (on-table ?x)
             (arm-empty)
             (holding ?x)
             (on ?x ?y))

(:action pickup
  :parameters (?ob)
  :precondition (and (clear ?ob) (on-table ?ob) (arm-empty))
  :effect (and (holding ?ob) (not (clear ?ob)) (not (on-table ?ob)) 
               (not (arm-empty))))

(:action putdown
  :parameters  (?ob)
  :precondition (holding ?ob)
  :effect (and (clear ?ob) (arm-empty) (on-table ?ob) 
               (not (holding ?ob))))

(:action stack
  :parameters  (?ob ?underob)
  :precondition (and (clear ?underob) (holding ?ob))
  :effect (and (arm-empty) (clear ?ob) (on ?ob ?underob)
               (not (clear ?underob)) (not (holding ?ob))))

(:action unstack
  :parameters  (?ob ?underob)
  :precondition (and (on ?ob ?underob) (clear ?ob) (arm-empty))
  :effect (and (holding ?ob) (clear ?underob)
               (not (on ?ob ?underob)) (not (clear ?ob)) (not (arm-empty)))))
"#;

pub const BLOCKSWORLD_PROBLEM13_TEXT: &str = r#"
(define (problem blocksworld-13)
(:domain blocksworld)
(:objects  b1 b2 b3 b4 - object)
(:init 
    (arm-empty)
    (clear b1)
    (on b1 b2)
    (on b2 b3)
    (on b3 b4)
    (on-table b4))

(:goal (and
    (clear b4)
    (on b4 b2)
    (on b2 b3)
    (on b3 b1)
    (on-table b1))))
"#;

pub const SPANNER_DOMAIN_TEXT: &str = r#"
; source => https://github.com/AI-Planning/pddl-generators/blob/main/spanner/domain.pddl
(define (domain spanner)
(:requirements :typing :strips)
(:types
	location locatable - object
	man nut spanner - locatable
)

(:predicates
	(at ?m - locatable ?l - location)
	(carrying ?m - man ?s - spanner)
	(usable ?s - spanner)
	(link ?l1 - location ?l2 - location)
	(tightened ?n - nut)
	(loose ?n - nut))

(:action walk
        :parameters (?start - location ?end - location ?m - man)
        :precondition (and (at ?m ?start)
                           (link ?start ?end))
        :effect (and (not (at ?m ?start)) (at ?m ?end)))

(:action pickup_spanner
        :parameters (?l - location ?s - spanner ?m - man)
        :precondition (and (at ?m ?l)
                           (at ?s ?l))
        :effect (and (not (at ?s ?l))
                     (carrying ?m ?s)))

(:action tighten_nut
        :parameters (?l - location ?s - spanner ?m - man ?n - nut)
        :precondition (and (at ?m ?l)
		      	   (at ?n ?l)
			   (carrying ?m ?s)
			   (usable ?s)
			   (loose ?n))
        :effect (and (not (loose ?n)) (not (usable ?s)) (tightened ?n)))
)
"#;

pub const SPANNER_PROBLEM10_TEXT: &str = r#"
;; spanners=4, nuts=2, locations=6, out_folder=testing/easy, instance_id=10, seed=1016

(define (problem spanner-10)
 (:domain spanner)
 (:objects 
    bob - man
    spanner1 spanner2 spanner3 spanner4 - spanner
    nut1 nut2 - nut
    shed location1 location2 location3 location4 location5 location6 gate - location
 )
 (:init 
    (at bob shed)
    (at spanner1 location3)
    (usable spanner1)
    (at spanner2 location6)
    (usable spanner2)
    (at spanner3 location4)
    (usable spanner3)
    (at spanner4 location5)
    (usable spanner4)
    (at nut1 gate)
    (loose nut1)
    (at nut2 gate)
    (loose nut2)
    (link shed location1)
    (link location6 gate)
    (link location1 location2)
     (link location2 location3)
     (link location3 location4)
     (link location4 location5)
     (link location5 location6)
 )
 (:goal  (and (tightened nut1)
   (tightened nut2))))
"#;
