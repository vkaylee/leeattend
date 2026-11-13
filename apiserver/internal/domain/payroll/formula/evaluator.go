package formula

import (
	"errors"
	"fmt"
	"strings"
	"unicode"
	"unicode/utf8"

	"github.com/shopspring/decimal"
)

var (
	ErrInvalidExpression = errors.New("payroll formula: invalid expression")
	ErrUnknownVariable   = errors.New("payroll formula: unknown variable")
	ErrUnknownFunction   = errors.New("payroll formula: unknown function")
	ErrDivisionByZero    = errors.New("payroll formula: division by zero")
	ErrResourceLimit     = errors.New("payroll formula: resource limit exceeded")
)

const (
	maxExpressionLength = 4096
	maxDepth             = 64
	maxSteps             = 10000
)

type Evaluator struct {
	values map[string]decimal.Decimal
	steps  int
	depth  int
}

func Evaluate(expression string, values map[string]decimal.Decimal) (decimal.Decimal, error) {
	if strings.TrimSpace(expression) == "" || len(expression) > maxExpressionLength {
		return decimal.Zero, ErrInvalidExpression
	}
	parser := parser{input: expression, values: values}
	result, err := parser.parseExpression()
	if err != nil {
		return decimal.Zero, err
	}
	parser.skipSpace()
	if parser.position != len(parser.input) {
		return decimal.Zero, ErrInvalidExpression
	}
	return result, nil
}

type parser struct {
	input    string
	position int
	values   map[string]decimal.Decimal
	steps    int
	depth    int
}

func (p *parser) parseExpression() (decimal.Decimal, error) {
	return p.parseComparison()
}

func (p *parser) parseComparison() (decimal.Decimal, error) {
	left, err := p.parseTerm()
	if err != nil {
		return decimal.Zero, err
	}
	p.skipSpace()
	operator := ""
	for _, candidate := range []string{"<=", ">=", "==", "!=", "<", ">"} {
		if strings.HasPrefix(p.input[p.position:], candidate) {
			operator = candidate
			p.position += len(candidate)
			break
		}
	}
	if operator == "" {
		return left, nil
	}
	right, err := p.parseTerm()
	if err != nil {
		return decimal.Zero, err
	}
	matched := false
	switch operator {
	case "<":
		matched = left.LessThan(right)
	case "<=":
		matched = left.LessThanOrEqual(right)
	case ">":
		matched = left.GreaterThan(right)
	case ">=":
		matched = left.GreaterThanOrEqual(right)
	case "==":
		matched = left.Equal(right)
	case "!=":
		matched = !left.Equal(right)
	}
	if matched {
		return decimal.NewFromInt(1), nil
	}
	return decimal.Zero, nil
}

func (p *parser) parseTerm() (decimal.Decimal, error) {
	left, err := p.parseFactor()
	if err != nil {
		return decimal.Zero, err
	}
	for {
		p.skipSpace()
		if p.position >= len(p.input) || (p.input[p.position] != '+' && p.input[p.position] != '-') {
			return left, nil
		}
		operator := p.input[p.position]
		p.position++
		right, err := p.parseFactor()
		if err != nil {
			return decimal.Zero, err
		}
		if operator == '+' {
			left = left.Add(right)
		} else {
			left = left.Sub(right)
		}
	}
}

func (p *parser) parseFactor() (decimal.Decimal, error) {
	left, err := p.parsePrimary()
	if err != nil {
		return decimal.Zero, err
	}
	for {
		p.skipSpace()
		if p.position >= len(p.input) || (p.input[p.position] != '*' && p.input[p.position] != '/') {
			return left, nil
		}
		operator := p.input[p.position]
		p.position++
		right, err := p.parsePrimary()
		if err != nil {
			return decimal.Zero, err
		}
		if operator == '*' {
			left = left.Mul(right)
			continue
		}
		if right.IsZero() {
			return decimal.Zero, ErrDivisionByZero
		}
		left = left.Div(right)
	}
}

func (p *parser) parsePrimary() (decimal.Decimal, error) {
	p.steps++
	if p.steps > maxSteps {
		return decimal.Zero, ErrResourceLimit
	}
	p.skipSpace()
	if p.position >= len(p.input) {
		return decimal.Zero, ErrInvalidExpression
	}
	if p.input[p.position] == '+' || p.input[p.position] == '-' {
		sign := p.input[p.position]
		p.position++
		value, err := p.parsePrimary()
		if sign == '-' {
			value = value.Neg()
		}
		return value, err
	}
	if p.input[p.position] == '(' {
		if p.depth >= maxDepth {
			return decimal.Zero, ErrResourceLimit
		}
		p.depth++
		p.position++
		value, err := p.parseExpression()
		p.skipSpace()
		if p.position >= len(p.input) || p.input[p.position] != ')' {
			return decimal.Zero, ErrInvalidExpression
		}
		p.position++
		p.depth--
		return value, err
	}
	if unicode.IsDigit(rune(p.input[p.position])) || p.input[p.position] == '.' {
		return p.parseNumber()
	}
	name := p.parseName()
	if name == "" {
		return decimal.Zero, ErrInvalidExpression
	}
	p.skipSpace()
	if p.position < len(p.input) && p.input[p.position] == '(' {
		return p.parseFunction(name)
	}
	value, ok := p.values[name]
	if !ok {
		return decimal.Zero, fmt.Errorf("%w: %s", ErrUnknownVariable, name)
	}
	return value, nil
}

func (p *parser) parseNumber() (decimal.Decimal, error) {
	start := p.position
	dot := false
	for p.position < len(p.input) {
		character := p.input[p.position]
		if character == '.' {
			if dot {
				break
			}
			dot = true
			p.position++
			continue
		}
		if character < '0' || character > '9' {
			break
		}
		p.position++
	}
	value, err := decimal.NewFromString(p.input[start:p.position])
	if err != nil {
		return decimal.Zero, ErrInvalidExpression
	}
	return value, nil
}

func (p *parser) parseName() string {
	start := p.position
	for p.position < len(p.input) {
		character := rune(p.input[p.position])
		if !(unicode.IsLetter(character) || unicode.IsDigit(character) || character == '_' || character == '.') {
			break
		}
		p.position++
	}
	return p.input[start:p.position]
}

func (p *parser) parseFunction(name string) (decimal.Decimal, error) {
	p.position++
	args := make([]decimal.Decimal, 0, 3)
	p.skipSpace()
	if p.position < len(p.input) && p.input[p.position] != ')' {
		for {
			value, err := p.parseExpression()
			if err != nil {
				return decimal.Zero, err
			}
			args = append(args, value)
			p.skipSpace()
			if p.position >= len(p.input) || p.input[p.position] == ')' {
				break
			}
			if p.input[p.position] != ',' {
				return decimal.Zero, ErrInvalidExpression
			}
			p.position++
		}
	}
	if p.position >= len(p.input) || p.input[p.position] != ')' {
		return decimal.Zero, ErrInvalidExpression
	}
	p.position++
	switch name {
	case "min":
		return aggregate(args, func(a, b decimal.Decimal) decimal.Decimal { if a.LessThan(b) { return a }; return b })
	case "max":
		return aggregate(args, func(a, b decimal.Decimal) decimal.Decimal { if a.GreaterThan(b) { return a }; return b })
	case "round":
		if len(args) != 2 || !args[1].IsInteger() || args[1].IsNegative() || args[1].GreaterThan(decimal.NewFromInt(6)) {
			return decimal.Zero, ErrInvalidExpression
		}
		return args[0].Round(int32(args[1].IntPart())), nil
	case "clamp":
		if len(args) != 3 {
			return decimal.Zero, ErrInvalidExpression
		}
		if args[1].GreaterThan(args[2]) {
			return decimal.Zero, ErrInvalidExpression
		}
		if args[0].LessThan(args[1]) { return args[1], nil }
		if args[0].GreaterThan(args[2]) { return args[2], nil }
		return args[0], nil
	case "if":
		if len(args) != 3 { return decimal.Zero, ErrInvalidExpression }
		if !args[0].IsZero() { return args[1], nil }
		return args[2], nil
	default:
		return decimal.Zero, fmt.Errorf("%w: %s", ErrUnknownFunction, name)
	}
}

func aggregate(args []decimal.Decimal, combine func(decimal.Decimal, decimal.Decimal) decimal.Decimal) (decimal.Decimal, error) {
	if len(args) == 0 { return decimal.Zero, ErrInvalidExpression }
	result := args[0]
	for _, value := range args[1:] { result = combine(result, value) }
	return result, nil
}

func (p *parser) skipSpace() {
	for p.position < len(p.input) {
		character, size := utf8.DecodeRuneInString(p.input[p.position:])
		if !unicode.IsSpace(character) {
			return
		}
		p.position += size
	}
}
