This doc highlight the assumption we have on the espresso 


The espresso provides two roles: 
- For each regional finalized L2 half block, it provides a timestamp, ordered, public bulletin for every replica cross-region. 



You can see my part 0 of the formalization v5, and v6 

We assume the 


The GFR looks very simple 

- Given finalized L1 logs and Espresso logs 
    - we first pick a canonical finalized physical blocks. 
        - From genesis: 
            - encouter a resume: 
            - encouter a force: 
    - then we merge with deposit only. 