+       # Set cell 0 to 1
[       # Start of first loop (level 1)
  ++++    # Increment cell 1 by 4
  [       # Start of second loop (level 2)
    ++      # Increment cell 2 by 2
    [       # Start of third loop (level 3)
      +++     # Increment cell 3 by 3
      [       # Start of fourth loop (level 4)
        ----    # Decrement cell 4 by 4
        [       # Start of fifth loop (level 5)
          ++++    # Increment cell 5 by 4
          [       # Start of sixth loop (level 6)
            ------  # Decrement cell 6 by 6
            [       # Start of seventh loop (level 7)
              ++++++++    # Increment cell 7 by 8
              [       # Start of eighth loop (level 8)
                --------    # Decrement cell 8 by 8
                [       # Start of ninth loop (level 9)
                  ++++++++    # Increment cell 9 by 8
                  [       # Start of tenth loop (level 10)
                    --------    # Decrement cell 10 by 8
                    ++++++++++---------+
                    ++++++++++++++++++++
                    ++++++++++++++++++++
                    ++++++++++++++++++++
                    ++++++++++++++++++++
                    ++++++++++++++++++++
                    ++++++++++++++++++
                    ++++++++
                    .       # Output the current cell value
                  ]   # End of tenth loop
                  -   # Decrement cell 9
                ]   # End of ninth loop
                -   # Decrement cell 8
              ]   # End of eighth loop
              -   # Decrement cell 7
            ]   # End of seventh loop
            -   # Decrement cell 6
          ]   # End of sixth loop
          -   # Decrement cell 5
        ]   # End of fifth loop
        -   # Decrement cell 4
      ]   # End of fourth loop
      -   # Decrement cell 3
    ]   # End of third loop
    -   # Decrement cell 2
  ]   # End of second loop
  -   # Decrement cell 1
]   # End of first loop