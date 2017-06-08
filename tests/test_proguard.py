from libsourcemap import ProguardView


def test_basics():
    with open('tests/fixtures/mapping.txt', 'rb') as f:
        mapping = f.read()

    view = ProguardView.from_bytes(mapping)
    assert view.has_line_info

    assert view.lookup('android.support.constraint.ConstraintLayout$a') \
        == 'android.support.constraint.ConstraintLayout$LayoutParams'

    assert view.lookup('android.support.constraint.a.b:a', 116) \
        == 'android.support.constraint.solver.ArrayRow:createRowDefinition'


def test_basics():
    view = ProguardView.from_path('tests/fixtures/mapping.txt')

    assert view.has_line_info

    assert view.lookup('android.support.constraint.ConstraintLayout$a') \
        == 'android.support.constraint.ConstraintLayout$LayoutParams'

    assert view.lookup('android.support.constraint.a.b:a', 116) \
        == 'android.support.constraint.solver.ArrayRow:createRowDefinition'
